#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};
use serde::{Deserialize, Serialize};

fn main() {
    let context = tauri::generate_context!();
    tauri::Builder::default()
        .menu(tauri::Menu::os_default(&context.package_info().name))
        .invoke_handler(tauri::generate_handler![register_nanaco_gift,])
        .run(context)
        .expect("error while running tauri application");
}

struct NanacoTab {
    tab: Arc<Tab>,
}

impl NanacoTab {
    fn input_nanaco_number(&mut self, nanaco_number: &str) -> Result<(), failure::Error> {
        let nanaco_number_element = self.tab.wait_for_element(
            "#login_password > table > tbody > tr:nth-child(1) > td.centerCell > input",
        );

        match nanaco_number_element {
            Ok(element) => {
                element.click()?;
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }

        self.tab.type_str(nanaco_number)?;

        Ok(())
    }

    fn input_password(&mut self, password: &str) -> Result<(), failure::Error> {
        self.tab
            .wait_for_element(
                "#login_password > table > tbody > tr:nth-child(2) > td.centerCell > input",
            )?
            .click()?;
        self.tab.type_str(password)?;
        Ok(())
    }

    fn login(&mut self) -> Result<(), failure::Error> {
        self.tab
            .wait_for_element("#login_password > div.login_box > input[type=image]")?
            .click()?;
        Ok(())
    }

    fn switch_to_gift_register_page(&mut self) -> Result<(), failure::Error> {
        let register_gift_navigation = self.tab.wait_for_element("#memberNavi02 > a");

        match register_gift_navigation {
            Ok(register_gift_navigation) => {
                register_gift_navigation.click()?;
            }
            Err(_e) => {
                let text = self
                    .tab
                    .wait_for_element("#box2Right")?
                    .call_js_fn("function() { return this.innerText }", false)?
                    .value
                    .unwrap()
                    .to_string();
                return Err(failure::err_msg(text));
                // return Err(text);
            }
        }
        self.tab.wait_for_element("#register > form")?.call_js_fn("function() { this.setAttribute('target', '_self'); this.setAttribute('onsubmit', '\"\"') }", false)?;

        self.tab
            .wait_for_element("#register > form > p > input[type=image]")?
            .click()?;
        Ok(())
    }

    fn input_gift_id(&mut self, gift_id: &str) -> Result<(), failure::Error> {
        let gift_id_forms = self.tab.wait_for_elements("input[id^='gift0']");

        match gift_id_forms {
            Ok(gift_id_forms) => {
                if gift_id.len() == 16 {
                    let gift_id_input = vec![
                        &gift_id[0..4],
                        &gift_id[4..8],
                        &gift_id[8..12],
                        &gift_id[12..],
                    ];
                    for (gift_id_form, gift) in gift_id_forms.iter().zip(gift_id_input.iter()) {
                        gift_id_form.click().expect("Fail to click element");
                        self.tab.type_str(gift).expect("Fail to type gift id");
                    }
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
        Ok(())
    }

    fn register_gift_id(&mut self) -> Result<(), failure::Error> {
        self.tab.wait_for_element("#submit-button")?.click()?;

        let register_buttun = self
            .tab
            .wait_for_element("#nav2Next > input[type=image]:nth-child(2)");
        match register_buttun {
            Ok(buttun) => {
                buttun.click()?;
                let finish_element = self.tab.wait_for_element("#navNext > a > img");
                match finish_element {
                    Ok(_finish_element) => {}
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                return Err(e);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct NanacoMessage {
    number: String,
    password: String,
    gift: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct NanacoResult {
    gift: String,
    result: String,
    message: String,
}

#[tauri::command]
fn register_nanaco_gift(message: NanacoMessage) -> Result<NanacoResult, String> {
    let options = LaunchOptionsBuilder::default()
        .window_size(Some((200, 200)))
        .headless(false)
        .build()
        .expect("Fail to build");

    let browser = Browser::new(options).expect("Fail to create browser");
    let tab = browser
        .wait_for_initial_tab()
        .expect("Fail to wait for initial tab");
    tab.navigate_to("https://www.nanaco-net.jp/pc/emServlet")
        .expect("Fail to navigate to nanaco");
    tab.wait_until_navigated()
        .expect("Fail to wait for navigation");

    let mut nanaco_tab = NanacoTab { tab: tab };

    nanaco_tab
        .input_nanaco_number(&message.number)
        .expect("Fail to input nanaco number");
    nanaco_tab
        .input_password(&message.password)
        .expect("Fail to input password");
    nanaco_tab.login().expect("Fail to login");
    match nanaco_tab.switch_to_gift_register_page() {
        Ok(_) => {}
        Err(e) => {
            println!("{:?}", e);
            return Ok(NanacoResult {
                gift: message.gift.clone(),
                result: "Fail".to_string(),
                message: "ログインに失敗しました".to_string(),
            });
        }
    }
    nanaco_tab
        .input_gift_id(&message.gift)
        .expect("Fail to input gift id");
    match nanaco_tab.register_gift_id() {
        Ok(_) => {}
        Err(e) => {
            println!("{:?}", e);
            return Ok(NanacoResult {
                gift: message.gift.clone(),
                result: "Fail".to_string(),
                message: "ギフト番号が正しくないか、有効ではありません".to_string(),
            });
        }
    }

    Ok(NanacoResult {
        gift: message.gift.clone(),
        result: "Success".to_string(),
        message: "成功".to_string(),
    })
}
