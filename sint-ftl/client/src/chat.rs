use crate::state::GameContext;
use leptos::prelude::*;
use sint_core::{Action, GameAction};

#[component]
pub fn ChatView(ctx: GameContext) -> impl IntoView {
    let state = ctx.state;
    let pid = ctx.player_id.clone();
    let ctx_send = ctx.clone();

    let (input_value, set_input_value) = signal(String::new());

    let send_message = move || {
        let msg = input_value.get();
        if !msg.is_empty() {
            ctx_send
                .perform_action
                .call(Action::Game(GameAction::Chat { message: msg }));
            set_input_value.set(String::new());
        }
    };

    let send_on_enter = send_message.clone();
    let send_on_click = send_message.clone();

    view! {
        <div style="background: #222; border: 1px solid #444; border-radius: 8px; display: flex; flex-direction: column; height: 100%; box-sizing: border-box;">
            <div style="flex: 1; overflow-y: auto; padding: 10px; display: flex; flex-direction: column; gap: 8px;">
                {move || {
                    state
                        .get()
                        .chat_log
                        .into_iter()
                        .map(|msg| {
                            let is_me = msg.sender == pid;
                            let align = if is_me {
                                "align-self: flex-end; background: #3f51b5;"
                            } else {
                                "align-self: flex-start; background: #444;"
                            };

                            view! {
                                <div style=format!(
                                    "padding: 8px 12px; border-radius: 12px; max-width: 80%; color: white; {}",
                                    align,
                                )>
                                    <div style="font-size: 0.7em; opacity: 0.7; margin-bottom: 2px;">
                                        {msg.sender}
                                    </div>
                                    <div>{msg.text}</div>
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>

            <div style="padding: 10px; border-top: 1px solid #444; display: flex; gap: 10px;">
                <input
                    type="text"
                    placeholder="Type a message..."
                    style="flex: 1; padding: 8px; border-radius: 4px; border: 1px solid #555; background: #333; color: white;"
                    prop:value=input_value
                    on:input=move |ev| set_input_value.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            send_on_enter();
                        }
                    }
                />
                <button
                    style="padding: 8px 16px; background: #2196f3; border: none; color: white; border-radius: 4px; cursor: pointer;"
                    on:click=move |_| send_on_click()
                >
                    "Send"
                </button>
            </div>
        </div>
    }
}
