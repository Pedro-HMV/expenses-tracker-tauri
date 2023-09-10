use gloo_dialogs::alert;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[derive(Serialize, Deserialize)]
struct Content {
    expenses: Vec<Expense>,
    net_worth: f32,
    income: f32,
}

#[derive(Serialize, Deserialize, Clone)]
struct Expense {
    name: String,
    cost: f32,
    paid: bool,
    due_date: u32,
}

#[derive(Serialize, Deserialize)]
struct ContentArgs<'a> {
    content: &'a str,
}

#[function_component(AppData)]
pub fn get_expenses() -> Html {
    let app_data = use_state(|| Content {
        expenses: Vec::new(),
        net_worth: 0.0,
        income: 0.0,
    });

    let income_input_ref = use_node_ref();

    {
        let app_data = app_data.clone();
        use_effect(move || {
            spawn_local(async move {
                let args = to_value(&ContentArgs {
                    content: "expenses.json",
                })
                .unwrap();
                let content = invoke("get_content", args).await;
                let new_data: Content = serde_wasm_bindgen::from_value(content).unwrap();
                app_data.set(new_data);
            });
            || {}
        });
    }

    {
        let app_data = app_data.clone();
        let income = app_data.income.clone();
        let income2 = app_data.income.clone();
        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    if income == 0.0 {
                        return;
                    }
                    let args = to_value(&income).unwrap();
                    let success = invoke("edit_income", args).await;
                    let success: Result<f32, String> =
                        serde_wasm_bindgen::from_value(success).unwrap();
                    match success {
                        Ok(x) => app_data.set(Content {
                            expenses: app_data.expenses.clone(),
                            net_worth: app_data.net_worth.clone(),
                            income: x,
                        }),
                        _ => alert("Erro ao editar a renda."),
                    }
                })
            },
            income2,
        )
    }

    let edit_income = {
        let income_input_ref = income_input_ref.clone();
        let app_data = app_data.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let income = income_input_ref
                .cast::<web_sys::HtmlInputElement>()
                .unwrap()
                .value();
            app_data.set(Content {
                expenses: app_data.expenses.clone(),
                net_worth: app_data.net_worth.clone(),
                income: income.parse::<f32>().unwrap(),
            });
        })
    };

    html! {
        <main class="container">
        <div class="expenses-container">
            <ul class="expense-titles expense-column">
                <li class="expense-headings">{ "Título" }</li>
                { for app_data.expenses.iter().map(|expense| html! {
                    <li>{ expense.name.as_str() }</li>
                })
                }
            </ul>
            <ul class="expense-costs expense-column">
                <li class="expense-headings">{ "Custo"}</li>
                {for app_data.expenses.iter().map(|expense| html! {
                    <li>{ expense.cost }</li>
                })}
            </ul>
            <ul class="expense-dates expense-column">
                <li class="expense-headings">{ "Dia do vencimento"}</li>
                {for app_data.expenses.iter().map(|expense| html! {
                    <li>{ expense.due_date }</li>
                })}
            </ul>
            <ul class="expense-paids expense-column">
                <li class="expense-headings">{ "Pago?"}</li>
                {for app_data.expenses.iter().map(|expense| html! {
                    <li>{ expense.paid }</li>
                })}
            </ul>
            </div>

            <div>
                <form class="row" onsubmit={edit_income}>
                <input id="income-input" ref={income_input_ref} placeholder="Renda" type="text" value={app_data.income.to_string()}/>
                <button type="submit">{"Salvar"}</button>
            </form>

                <p>{ "Patrimônio líquido: " }{ app_data.net_worth }</p>
            </div>



        </main>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let greet_input_ref = use_node_ref();

    let name = use_state(|| String::new());

    let greet_msg = use_state(|| String::new());
    {
        let greet_msg = greet_msg.clone();
        let name = name.clone();
        let name2 = name.clone();
        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    if name.is_empty() {
                        return;
                    }

                    let args = to_value(&GreetArgs { name: &*name }).unwrap();
                    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
                    let new_msg = invoke("greet", args).await.as_string().unwrap();
                    greet_msg.set(new_msg);
                });

                || {}
            },
            name2,
        );
    }

    let greet = {
        let name = name.clone();
        let greet_input_ref = greet_input_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            name.set(
                greet_input_ref
                    .cast::<web_sys::HtmlInputElement>()
                    .unwrap()
                    .value(),
            );
        })
    };

    html! {
        <main class="container">
            <div class="row">
                <a href="https://tauri.app" target="_blank">
                    <img src="public/tauri.svg" class="logo tauri" alt="Tauri logo"/>
                </a>
                <a href="https://yew.rs" target="_blank">
                    <img src="public/yew.png" class="logo yew" alt="Yew logo"/>
                </a>
            </div>

            <p>{"Click on the Tauri and Yew logos to learn more."}</p>

            <p>
                {"Recommended IDE setup: "}
                <a href="https://code.visualstudio.com/" target="_blank">{"VS Code"}</a>
                {" + "}
                <a href="https://github.com/tauri-apps/tauri-vscode" target="_blank">{"Tauri"}</a>
                {" + "}
                <a href="https://github.com/rust-lang/rust-analyzer" target="_blank">{"rust-analyzer"}</a>
            </p>

            <form class="row" onsubmit={greet}>
                <input id="greet-input" ref={greet_input_ref} placeholder="Enter a name..." />
                <button type="submit">{"Greet"}</button>
            </form>

            <p><b>{ &*greet_msg }</b></p>
        </main>
    }
}
