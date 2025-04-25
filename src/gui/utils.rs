pub fn async_error_popup(
    prompt: &str,
) -> impl std::future::Future<Output = rfd::MessageDialogResult> + use<> {
    rfd::AsyncMessageDialog::new()
        .set_title("Error")
        .set_buttons(rfd::MessageButtons::Ok)
        .set_description(prompt)
        .show()
}

pub fn error_popup(prompt: &str) {
    rfd::MessageDialog::new()
        .set_title("Error")
        .set_buttons(rfd::MessageButtons::Ok)
        .set_description(prompt)
        .show();
}

pub fn error_chain_string(error: anyhow::Error) -> String {
    let mut message = String::new();

    let mut chain_iter = error.chain();

    message += &format!("Error: {}\n", chain_iter.next().unwrap());

    for err in chain_iter {
        message += &format!("\nCaused by:\n\t{}", err);
    }

    message
}
