# DiceRoll (Solana)

### Provable Dice Roll Program on Solana
The app consists of a vendor and a player. 
1. Player triggers a dice roll game by selecting side count of dice and amount of SOL to bet. 
2. Vendor creates a DiceRoll game on-chain and saves the bet of the user and sends the equal amount of SOL to DiceRoll PDA. In this step, vendor generates a random seed to be used in the dice roll.
3. When the DiceRoll account is created, user generates a random seed and sends approves the Play transaction by sending the bet amount to DiceRoll PDA.
4. Play function will combine the vendor hash, player hash and the current timestamp to roll the dice. This way, not a single participant can influence the result as the seeds are aggregated from different sources.
5. DiceRoll account sends the amount to the winner, logs the events and closes the account.
6. Players play until dices are not equal and player with the highest dice value wins.


#### Webapp
Webapp is generated using Next.js and [Solana Wallet Adapter](https://github.com/solana-labs/wallet-adapter#readme).

#### Running the app
1. Start the local Solana validator
2. Deploy the app to local cluster with `anchor deploy`
3. Run tests with "anchor test"
