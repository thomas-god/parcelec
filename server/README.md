# API Documentation

The following describes the various routes composing the parcelec API. 

### User related route
- [GET /scenarios](#List-all-available-scenarios)
- [GET /scenario/:scenario_id](#Get-information-about-a-scenario)
- [GET /sessions/open](#List-all-opened-session-that-can-be-joined)
- [PUT /session](#Open-a-new-session-(session_name-must-be-unique))
- [GET /session/:session_id](#Get-public-information-about-a-session)
- [PUT /session/:session_id/register_user](#Add-a-new-user-to-a-game-session)
- [PUT /session/:session_id/user/:user_id/ready](#Set-a-user-a-ready-to-start-a-game)
- [GET /session/:session_id/user/:user_id](#Get-informations-about-an-user)
- [GET /session/:session_id/user/:user_id/portfolio](#Get-a-user's-portfolio)
- [GET /session/:session_id/user/:user_id/conso](#Get-the-consumption-value)
- [GET /session/:session_id/user/:user_id/conso_forecast](#Get-the-consumption-forecast)
- [POST /session/:session_id/user/:user_id/bid](#Put-a-bid-to-the-market)
- [GET /session/:session_id/user/:user_id/bids](#Get-all-user's-bids)
- [DELETE /session/:session_id/user/:user_id/bid/:bid_id](#Delete-a-bid)
- [PUT /session/:session_id/user/:user_id/planning](#Put-a-user's-production-planning)
- [GET /session/:session_id/clearing](#Get-information-on-the-market-clearing)
- [GET /session/:session_id/user/:user_id/clearing](#Get-specific-information-on-a-user-energy-exchanges-following-market-clearing)
- [GET /session/:session_id/user/:user_id/results](#Get-energy-and-financial-results-when-a-phase-is-finished)
- [GET /session/:session_id/clearing/?user_id](#Get-all-the-bids-anonymously-after-a-session-has-cleared)
- [POST /session/:session_id/user_ir/:user_id/otc](#Post-an-over-the-counter-(OTC)-offer)
- [GET /session/:session_id/user_ir/:user_id/otc](#Get-all-user's-OTCs-(send-and-received))
- [PUT /session/:session_id/user/:user_id/otc/:otc_id/accept](#Accept-an-OTC)
- [PUT /session/:session_id/user/:user_id/otc/:otc_id/reject](#Reject-an-OTC)


## Game session related routes

### List all available scenarios
 - Route: `GET /scenarios`
 - Response : 
    - Type : `application/json`,
    - Body : 
    ``` js
      [
        {
          id: UUID string,
          name: string,
          description: string,
          difficulty: 'easy' | 'medium' | 'hard',
          multi_game: boolean
        }
      ]
    ```

### Get information about a scenario
 - Route: `GET /scenario/:scenario_id`
 - Response : 
    - Type : `application/json`,
    - Body : 
    ``` js
      {
        options: {},
        portfolio: []
      }
    ```
### List all opened session that can be joined
 - Route: `GET /sessions/open`
 - Response : 
    - Type : `application/json`,
    - Body : 
    ``` js
      [
        {
          session_id: UUID string,
          name: string
        }
      ]
    ```

### Open a new session (session_name must be unique)
 - Route: `PUT /session`
    - Type : `application/json`,
    - Body : 
    ```js
      {
        session_name: string,
        scenario_id?: string
      }
    ```
 - Response : 
    - Code : `201` on success
    - Type : `application/json`,
    - Body : 
    ``` js
      [
        {
          id: UUID string,
          name: string,
          status: 'open' | 'running' | 'closed'
        }
      ]
    ```
    - `400` if a session already exists with this name

### Get public information about a session
- Route : `GET /session/:session_id`
- Response :
    - Code `200` on success
    - Type : `application/json`,
    - Body : 
    ```js
      {
        session_id: UUID string,
        name: string,
        status: 'open' | 'running' | 'closed',
        multi_game: boolean,
        users: [
          { name: string }
        ],
        bids_allowed: boolean,
        clearing_available: boolean,
        plannings_allowed: boolean,
        results_available: boolean,
        phase_infos: {
          start_time: UTC string date,
          clearing_time: UTC string date,
          results_time: UTC string date,
        }
      }
    ```

## User related routes

### Add a new user to a game session
- Route : `PUT /session/:session_id/register_user`
- Response :
    - Code : `201` on success
    - Type : `application/json`,
    - Body : 
    ``` js
      { user_id: UUID string }
    ```
    - `400` on failure

### Set a user a ready to start a game
- Route : `PUT /session/:session_id/user/:user_id/ready`
- Response :
    - `201` on success
    - `400` on failure

### Get informations about an user
- Route : `GET /session/:session_id/user/:user_id`
- Response :
    - Code : `200` on success
    - Type : `application/json`,
    - Body : 
    ``` js
      {
        session_id: UUID string,
        name: string,
        ready: boolean
      }
    ```

## Game related routes

### Get a user's portfolio
- Route : `GET /session/:session_id/user/:user_id/portfolio`
- Response :
    - Code : `200` on success
    - Type : `application/json`,
    - Body : 
    ``` js
      {
        id: UUID string,
        session_id: UUID string,
        user_id: UUID string,
        type: "nuc" | "therm" | "hydro" | "ren" | "storage",
        p_min_mw: number,
        p_max_mw: number,
        stock_max_mwh: number,
        price_eur_per_mwh: number,
        planning: number
      }
    ```

### Get the consumption value
- Route : `GET /session/:session_id/user/:user_id/conso`
- Response :
    - Code : `200` on success
    - Type : `application/json`,
    - Body : 
    ``` js
      { conso_mw: number }
    ```
### Get the consumption forecast
- Route : `GET /session/:session_id/user/:user_id/conso_forecast`
- Response :
    - Code : `200` on success
    - Type : `application/json`,
    - Body : 
    ``` js
      { number[] }
    ```

### Put a bid to the market
- Route : `POST /session/:session_id/user/:user_id/bid`
    - Type : `application/json`,
    - Body : 
    ``` js
      {
        bid: {
          type: 'sell' | 'buy',
          volume_mwh: number,
          price_eur_per_mwh: number
        }
      }
    ```
- Response :
    - Code : `201` on success
    - Type : `application/json`,
    - Body : 
    ``` js
      {
        bid_id: UUID string
      }
    ```

### Get all user's bids
- Route : `GET /session/:session_id/user/:user_id/bids`
- Response :
    - Type : `application/json`,
    - Body : 
    ``` js
      [
        {
          id: UUID string, 
          user_id: UUID string, 
          session_id: UUID string, 
          phase_no: number, 
          type: 'sell' | 'buy',
          volume_mwh: number,
          price_eur_per_mwh: number
        }
      ]
    ```

### Delete a bid
- Route : `DELETE /session/:session_id/user/:user_id/bid/:bid_id`
- Response :
    - Code : `200` on success

### Put a user's production planning
- Route : `PUT /session/:session_id/user/:user_id/planning`
    - Type : `application/json`,
    - Body : 
    ``` js
      {
        user_id: UUID string;
        session_id: UUID string;
        plant_id: UUID string;
        p_mw: number;
      }
    ```
- Response :
    - Code : `201` on success

### Get information on the market clearing
- Route : `GET /session/:session_id/clearing`
- Response :
    - Type : `application/json`,
    - Body : 
    ``` js
      {
        phase_id: number;
        volume_mwh: number;
        price_eur_per_mwh: number;
      }
    ```

### Get specific information on a user energy exchanges following market clearing
- Route : `GET /session/:session_id/user/:user_id/clearing`
- Response :
    - Type : `application/json`,
    - Body : 
    ``` js
      [
        {
          type: "buy" | "sell";
          volume_mwh: number;
          price_eur_per_mwh: number;
        }
      ]
    ```

### Get energy and financial results when a phase is finished
- Route : `GET /session/:session_id/user/:user_id/results`
- Response :
    - Type : `application/json`,
    - Body : 
    ``` js
      {
        user_id: UUID string;
        session_id: UUID string;
        phase_no: number;
        conso_mwh: number;
        conso_eur: number;
        prod_mwh: number;
        prod_eur: number;
        sell_mwh: number;
        sell_eur: number;
        buy_mwh: number;
        buy_eur: number;
        imbalance_mwh: number;
        imbalance_costs_eur: number;
        balance_eur: number;
      }
    ```

### Get all the bids anonymously after a session has cleared
- Route : `GET /session/:session_id/clearing/?user_id`
- Response :
    - Type : `application/json`,
    - Body : 
    ``` js
      [
        {
          type: "buy" | "sell";
          volume_mwh: number;
          price_eur_per_mwh: number;
          own_bid: boolean;
        }
      ]
    ```

### Get all user's OTCs (send and received)
- Route : `GET /session/:session_id/user_ir/:user_id/otc`
- Response :
    - Type : `application/json`,
    - Body : 
    ``` js
      [
        {
          id: UUID string;
          user_from: string;
          user_to: string;
          session_id: UUID string;
          phase_no: number;
          type: "buy" | "sell";
          volume_mwh: number;
          price_eur_per_mwh: number;
          status: "pending" | "accepted" | "rejected";
        }
      ]
    ```

### Post an over-the-counter (OTC) offer
- Route : `POST /session/:session_id/user_ir/:user_id/otc`
    - Type : `application/json`,
    - Body : 
    ``` js
      {
        type: "buy" | "sell",
        user_to: string,
        volume_mwh: number,
        price_eur_per_mwh: number,
      }
    ```
- Response :
    - Type : `application/json`,
    - Code : `201` on success
    - Body : 
    ``` js
      {
        otc_id: UUID string;
      }
    ```

### Accept an OTC
- Route : `PUT /session/:session_id/user/:user_id/otc/:otc_id/accept`
- Response :
    - Code : `200` on success

### Reject an OTC 
- Route : `PUT /session/:session_id/user/:user_id/otc/:otc_id/reject`
- Response :
    - Code : `200` on success
