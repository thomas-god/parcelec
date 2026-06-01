#![allow(dead_code, unused_imports)]
use common::{DEFAULT_WAIT_TIMEOUT, init, init_webdriver_client};
use fantoccini::{Client, Locator, error::CmdError};

mod common;

#[cfg(all(test, feature = "e2e-tests"))]
#[tokio::test]
async fn test_run_tutorial() {
    let (addr, _) = init().await;
    let client = init_webdriver_client().await;
    let c = client.clone();
    let res = tokio::spawn(async move {
        c.goto(&addr).await.unwrap();

        let start_tutorial = c
            .wait()
            .at_most(DEFAULT_WAIT_TIMEOUT)
            .for_element(Locator::XPath("//button[contains(text(), 'Tutoriel')]"))
            .await
            .unwrap();
        start_tutorial.click().await.unwrap();

        assert_eq!(
            c.current_url().await.unwrap().as_ref(),
            format!("{}/tutorial", addr)
        );

        // Avoid depending of the navigation timing
        assert!(
            vec![format!("{}/tutorial", addr), format!("{}/game", addr)]
                .contains(&c.current_url().await.unwrap().as_ref().to_string())
        );

        // Close the tutorial popup
        c.wait()
            .at_most(DEFAULT_WAIT_TIMEOUT)
            .for_element(Locator::XPath(
                "//button[contains(text(), '✕')]",
            ))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();

        // Dispatch the battery and check the position updates
        let initial_position: isize = c
            .wait()
            .at_most(DEFAULT_WAIT_TIMEOUT * 5)
            .for_element(Locator::XPath("//span[@data-testid='portfolio-position']"))
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
            .strip_suffix(" MW")
            .unwrap()
            .parse()
            .unwrap();

        let battery_input = c
            .wait()
            .at_most(DEFAULT_WAIT_TIMEOUT)
            .for_element(Locator::XPath("//input[@data-testid='battery-input']"))
            .await
            .unwrap();

        c.execute(
            "
            const input = arguments[0];
            input.value = '50';
            input.dispatchEvent(new Event('input', {bubbles: true}));
            ",
            vec![serde_json::to_value(&battery_input).unwrap()],
        )
        .await
        .unwrap();
        let expected_position = initial_position - 50;
        c.wait()
            .at_most(DEFAULT_WAIT_TIMEOUT * 5)
            .for_element(Locator::XPath(&format!(
               "//span[@data-testid='portfolio-position' and contains(text(), '{expected_position} MW')]"
            )))
            .await
            .unwrap();

        // End the period
        c.wait()
            .at_most(DEFAULT_WAIT_TIMEOUT)
            .for_element(Locator::XPath(
                "//button[contains(text(), 'Terminer')]",
            ))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();

        // Scores should be displayed
        c.wait()
            .at_most(DEFAULT_WAIT_TIMEOUT)
            .for_element(Locator::XPath(&format!(
                "//th[contains(text(), '{expected_position} MW')]"
            )))
            .await
            .unwrap();

        // Start the next period
        c.find(Locator::XPath(
            "//button[contains(text(), 'Période suivante')]",
        ))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();
    })
    .await;
    client.close().await.unwrap();
    if let Err(e) = res {
        std::panic::resume_unwind(Box::new(e));
    }
}
