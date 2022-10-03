import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { DiceRoll } from "../target/types/dice_roll";
const { SystemProgram, LAMPORTS_PER_SOL } = anchor.web3;

import * as assert from "assert";
import { expect } from "chai";
const program = anchor.workspace.DiceRoll;

function programForUser(user) {
  return new anchor.Program(program.idl, program.programId, user.provider);
}

async function play(provider, program, diceRoll, playerOne, playerTwo) {
  const randomSeed = new anchor.BN(Math.floor(Math.random() * 100000));

  const tx = await program.rpc.play(randomSeed, {
    accounts: {
      vendor: playerOne.publicKey,
      player: playerTwo.publicKey,
      diceRoll,
      systemProgram: anchor.web3.SystemProgram.programId,
    },
    signers: playerTwo instanceof (anchor.Wallet as any) ? [] : [playerTwo],
  });

  const gameState = await program.account.diceRoll.fetch(diceRoll);
  console.log("playerTwo: ", playerTwo.publicKey.toString());
  console.log("Winner:", gameState.state.finished.winner.toString());
  console.log({ gameState: gameState.players });
  await provider.connection.confirmTransaction(tx);
}

describe("dice-roll", () => {
  const provider = anchor.Provider.local("http://127.0.0.1:8899");
  anchor.setProvider(provider);
  const player_setup_dice_side_count_choice = 5;
  it("setups the game", async () => {
    const vendor = anchor.web3.Keypair.generate();
    const player = anchor.web3.Keypair.generate();

    let sig = await provider.connection.requestAirdrop(player.publicKey, 1000000000000);
    await provider.connection.confirmTransaction(sig);

    let sig2 = await provider.connection.requestAirdrop(vendor.publicKey, 1000000000000);
    await provider.connection.confirmTransaction(sig2);

    const vendorProgram = programForUser(vendor);

    const [diceRollPDA, _] = await anchor.web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode("dice-roll"), vendor.publicKey.toBuffer(), player.publicKey.toBuffer()],
      program.programId
    );

    const betAmount = new anchor.BN(100000000000);
    const randomSeed = new anchor.BN(Math.floor(Math.random() * 100000));

    await vendorProgram.rpc.setup(player.publicKey, betAmount, player_setup_dice_side_count_choice, randomSeed, {
      accounts: {
        diceRoll: diceRollPDA,
        vendor: vendor.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [vendor],
    });

    const gameState = await program.account.diceRoll.fetch(diceRollPDA);
    expect(gameState.players[0].toString()).to.be.equal(vendor.publicKey.toString());
    expect(gameState.players[1].toString()).to.be.equal(player.publicKey.toString());
    expect(gameState.vendorSeed.toString()).to.be.equal(randomSeed.toString());
  });

  it("plays the game", async () => {
    const vendor = anchor.web3.Keypair.generate();
    const player = anchor.web3.Keypair.generate();

    let sig = await provider.connection.requestAirdrop(player.publicKey, 1000000000000);
    await provider.connection.confirmTransaction(sig);
    let sig2 = await provider.connection.requestAirdrop(vendor.publicKey, 1000000000000);
    await provider.connection.confirmTransaction(sig2);

    const vendorProgram = programForUser(vendor);
    const playerProgram = programForUser(player);

    const [diceRollPDA, _] = await anchor.web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode("dice-roll"), vendor.publicKey.toBuffer(), player.publicKey.toBuffer()],
      program.programId
    );

    const betAmount = new anchor.BN(50000000000);
    const randomSeed = new anchor.BN(Math.floor(Math.random() * 100000));

    await vendorProgram.rpc.setup(player.publicKey, betAmount, randomSeed, {
      accounts: {
        diceRoll: diceRollPDA,
        vendor: vendor.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [vendor],
    });

    await play(provider, playerProgram, diceRollPDA, vendor, player);

    const gameState = await program.account.diceRoll.fetch(diceRollPDA);

    expect(gameState.players[0].toString()).to.be.equal(vendor.publicKey.toString());
    expect(gameState.players[1].toString()).to.be.equal(player.publicKey.toString());
    expect(gameState.vendorSeed.toString()).to.be.equal(randomSeed.toString());

    const vendorBalanceAfterRoll = await provider.connection.getAccountInfo(vendor.publicKey);
    console.log("vendorBalanceAfterRoll", vendorBalanceAfterRoll);

    const playerBalanceAfterRoll = await provider.connection.getAccountInfo(player.publicKey);
    console.log("playerBalanceAfterRoll", playerBalanceAfterRoll);
  });
});
