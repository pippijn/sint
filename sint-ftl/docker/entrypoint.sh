#!/bin/bash

# Start the Backend Server in the background
echo "Starting sint-server on port 3000..."
/usr/local/bin/sint-server &

# Start Nginx in the foreground
echo "Starting Nginx..."
nginx -g "daemon off;"
