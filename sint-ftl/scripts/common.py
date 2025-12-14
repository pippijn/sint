import os
import sys
import subprocess
import time
from typing import Optional, Any, Union

# Configuration
# scripts/common.py -> ../scripts -> .. (ROOT)
ROOT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
VENV_DIR = os.path.join(ROOT_DIR, '.venv')
VENV_BIN = os.path.join(VENV_DIR, 'bin')
PYTHON_EXEC = os.path.join(VENV_BIN, 'python3')
BUILD_MARKER = os.path.join(VENV_DIR, '.build_marker')

def run_cmd(cmd: str, cwd: str = ROOT_DIR, env: Optional[dict[str, str]] = None, background: bool = False, quiet: bool = False) -> Any:
    if not quiet:
        print(f'🚀 Running: {cmd}')
    if env is None:
        env = os.environ.copy()
    
    # Ensure venv is in path
    env['PATH'] = f'{VENV_BIN}:{env["PATH"]}'
    
    if background:
        return subprocess.Popen(cmd, shell=True, cwd=cwd, env=env)
    else:
        ret = subprocess.call(cmd, shell=True, cwd=cwd, env=env)
        if ret != 0:
            print(f'❌ Command failed: {cmd}')
            sys.exit(ret)
        return ret

def install_requirements() -> None:
    print('📦 Synchronizing Python dependencies...')
    venv_python = os.path.join(VENV_BIN, 'python')
    run_cmd(f'{venv_python} -m pip install -q -r ai/requirements.txt', quiet=True)
    run_cmd(f'{venv_python} -m pip install -q maturin', quiet=True)

def check_venv() -> None:
    if not os.path.exists(VENV_DIR):
        print('🔧 Creating Python virtual environment...')
        subprocess.check_call([sys.executable, '-m', 'venv', '.venv'])
        install_requirements()

def get_latest_mtime(path: str) -> float:
    max_mtime = 0.0
    if os.path.isfile(path):
        return os.path.getmtime(path)
        
    for root, _, files in os.walk(path):
        for f in files:
            if f.endswith('.rs') or f.endswith('.toml') or f == 'Cargo.lock':
                p = os.path.join(root, f)
                m = os.path.getmtime(p)
                if m > max_mtime:
                    max_mtime = m
    return max_mtime

def needs_rebuild() -> bool:
    if not os.path.exists(BUILD_MARKER):
        return True
    
    last_build = os.path.getmtime(BUILD_MARKER)
    
    # Check Directories
    dirs_to_check = ['core', 'solver']
    for d in dirs_to_check:
        if get_latest_mtime(os.path.join(ROOT_DIR, d)) > last_build:
            return True

    # Check Workspace files
    if get_latest_mtime(os.path.join(ROOT_DIR, 'Cargo.toml')) > last_build:
        return True
    if os.path.exists(os.path.join(ROOT_DIR, 'Cargo.lock')):
        if get_latest_mtime(os.path.join(ROOT_DIR, 'Cargo.lock')) > last_build:
            return True
            
    return False

def generate_python_types() -> None:
    print('🧬 Generating Python Types...')
    schema_path = os.path.join(ROOT_DIR, 'schema.json')
    
    # Generate schema
    run_cmd(f'cargo run -p sint-core --example dump_schema > {schema_path}', quiet=True)
    
    # Generate types
    try:
        run_cmd(
            f'datamodel-codegen --input {schema_path} --input-file-type jsonschema '
            f'--output ai/game_types.py --target-python-version 3.10 '
            f'--use-standard-collections --output-model-type pydantic_v2.BaseModel '
            f'--use-annotated',
            quiet=True
        )
    except SystemExit:
        print('❌ Failed to generate Python types. Is datamodel-code-generator installed?')
        # Don't exit here, maybe the old types are still somewhat usable
    finally:
        if os.path.exists(schema_path):
            os.remove(schema_path)

def build_bindings(force: bool = False) -> None:
    if not force and not needs_rebuild():
        # print('⏩ Skipping build (up to date)')
        return

    print('🔨 Building Python Bindings...')
    # Skip generic cargo build --all, let maturin handle it
    run_cmd('maturin develop --release', cwd=os.path.join(ROOT_DIR, 'core'))
    run_cmd('maturin develop --release', cwd=os.path.join(ROOT_DIR, 'solver'))
    
    # Generate Python types from the new bindings
    generate_python_types()
    
    # Touch marker
    with open(BUILD_MARKER, 'w') as f:
        f.write(str(time.time()))
