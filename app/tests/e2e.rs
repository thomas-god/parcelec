#![allow(dead_code, unused_imports)]
use common::{DEFAULT_WAIT_TIMEOUT, init, init_webdriver_client};
use fantoccini::{Client, Locator, error::CmdError};

mod common;

async fn navigate_tutorial_steps(client: &Client) {
    // Introduction
    client
        .wait()
        .at_most(DEFAULT_WAIT_TIMEOUT)
        .for_element(Locator::XPath("//button[contains(text(), 'Introduction')]"))
        .await
        .unwrap();
    // Navigate tutorial steps with "»" buttons
    client
        .find(Locator::XPath("//button[contains(text(), '»')]"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();

    // Plants
    client
        .wait()
        .at_most(DEFAULT_WAIT_TIMEOUT)
        .for_element(Locator::XPath("//button[contains(text(), 'Centrales')]"))
        .await
        .unwrap();
    // Navigate tutorial steps with "»" buttons
    client
        .find(Locator::XPath("//button[contains(text(), '»')]"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();

    // Market
    client
        .wait()
        .at_most(DEFAULT_WAIT_TIMEOUT)
        .for_element(Locator::XPath("//button[contains(text(), 'Marché')]"))
        .await
        .unwrap();
    // Navigate tutorial steps with "»" buttons
    client
        .find(Locator::XPath("//button[contains(text(), '»')]"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();

    // Forecasts
    // Market
    client
        .wait()
        .at_most(DEFAULT_WAIT_TIMEOUT)
        .for_element(Locator::XPath("//button[contains(text(), 'prévisions')]"))
        .await
        .unwrap();
}

#[cfg(all(test, feature = "e2e-tests"))]
#[tokio::test]
async fn test_run_totorial() {
    let (addr, _) = init().await;
    let client = init_webdriver_client().await;
    let c = client.clone();
    let res = tokio::spawn(async move {
        c.goto(&addr).await.unwrap();

        let tutorial_nav = c
            .wait()
            .at_most(DEFAULT_WAIT_TIMEOUT)
            .for_element(Locator::XPath("//a[text()='📖 Tutoriel']"))
            .await
            .unwrap();
        tutorial_nav.click().await.unwrap();

        assert_eq!(
            c.current_url().await.unwrap().as_ref(),
            format!("{}/tutorial", addr)
        );

        navigate_tutorial_steps(&c).await;

        let start_tutorial = c
            .wait()
            .at_most(DEFAULT_WAIT_TIMEOUT)
            .for_element(Locator::XPath(
                "//button[contains(text(), 'Commencer une partie')]",
            ))
            .await
            .unwrap();
        start_tutorial.click().await.unwrap();

        assert_eq!(
            c.current_url().await.unwrap().as_ref(),
            format!("{}/game", addr)
        );

        // Dispatch the battery and check the positions updates
        c.wait()
            .at_most(DEFAULT_WAIT_TIMEOUT * 5)
            .for_element(Locator::XPath(
                "//div[contains(text(), 'Déficit : 900 MW')]",
            ))
            .await
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

        c.wait()
            .at_most(DEFAULT_WAIT_TIMEOUT)
            .for_element(Locator::XPath(
                "//div[contains(text(), 'Déficit : 950 MW')]",
            ))
            .await
            .unwrap();

        // End the period
        c.wait()
            .at_most(DEFAULT_WAIT_TIMEOUT)
            .for_element(Locator::XPath(
                "//button[contains(text(), 'Terminer la phase')]",
            ))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();

        // Scores should be displayed
        c.wait()
            .at_most(DEFAULT_WAIT_TIMEOUT)
            .for_element(Locator::XPath("//td[contains(text(), '-950 MW')]"))
            .await
            .unwrap();

        // Start the next period
        c.find(Locator::XPath(
            "//button[contains(text(), 'Phase suivante')]",
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
