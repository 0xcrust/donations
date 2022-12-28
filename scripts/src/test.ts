import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
  SystemProgram,
  LAMPORTS_PER_SOL
} from '@solana/web3.js';
import {
  getProgramKeypair,
  airdrop,
  campaignAccountDataLayout,
  CampaignLayout,
  mapCampaignLayout,
} from './utils';
import {assert} from 'chai';
import BN from "bn.js";

describe("crowdfunding", () => {
  const programKeypair: Keypair =  getProgramKeypair();

  const programId = programKeypair.publicKey;
  const connection = new Connection("http://localhost:8899", "confirmed");

  it("Simulates a fundraising campaign", async () => {
    const fundStarter = new Keypair();
    console.log("Fundstarter: ", fundStarter.publicKey.toBase58());
    await airdrop(connection, fundStarter, 2);

    const [campaignPDA, campaignBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("campaign", "utf8"), fundStarter.publicKey.toBuffer()], programId
    );
    console.log("campaignPDA: ", campaignPDA.toBase58());

    const [vaultPDA, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault", "utf8"), campaignPDA.toBuffer()], programId
    );
    console.log("vaultPDA: ", vaultPDA.toBase58());

    let expectedTarget = new BN(3 * LAMPORTS_PER_SOL);
    let expectedDescription = "Raising funds for a new Macbook for school";

    let instruction = 0;
    let targetBuffer = expectedTarget.toBuffer('le', 8);
    let descriptionBuf = Buffer.alloc(200);
    descriptionBuf.write(expectedDescription, 0, 200, 'utf8');

    let instructionData = Buffer.from([instruction]);
    instructionData = Buffer.concat([instructionData, targetBuffer, descriptionBuf]);

    const initCampaignTx = new TransactionInstruction({
      programId: programId,
      keys: [
        {
          pubkey: fundStarter.publicKey, 
          isSigner: true, 
          isWritable: true
        },
        {
          pubkey: campaignPDA,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: vaultPDA,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: SystemProgram.programId,
          isSigner: false,
          isWritable: false
        }
      ],
      data: instructionData
    });
  
    const tx = new Transaction().add(initCampaignTx);
    console.log("Sending initialize campaign transaction...");
    await sendAndConfirmTransaction(connection, tx, [fundStarter]);
    
    let campaignAccountInfo = await connection.getAccountInfo(campaignPDA);
    let campaignState = campaignAccountInfo.data;
    let decodedCampaignState = campaignAccountDataLayout
      .decode(campaignState) as CampaignLayout;

    let mappedCampaign = mapCampaignLayout(decodedCampaignState);
    /**
     * Assert that initial campaign state is as expected
     */
    assert.equal(mappedCampaign.isInitialized, true);
    assert.ok(mappedCampaign.authority.equals(fundStarter.publicKey));
    assert.ok(mappedCampaign.vault.equals(vaultPDA));
    assert.ok(mappedCampaign.description, expectedDescription);
    assert.equal(mappedCampaign.target.toNumber(), expectedTarget.toNumber());
    assert.equal(mappedCampaign.amountRaised.toNumber(), 0);
    assert.equal(mappedCampaign.bump, campaignBump);

    async function donate(
      campaign: PublicKey, 
      vault: PublicKey, 
      donator: Keypair, 
      amount: number
      ) {
      const amountBuffer = new BN(amount * LAMPORTS_PER_SOL).toBuffer('le', 8);

      const instruction = 1;
      let instructionData = Buffer.from([instruction]);
      instructionData = Buffer.concat([instructionData, amountBuffer]);

      let donateIx = new TransactionInstruction({
        programId: programId,
        keys: [
          {
            pubkey: donator.publicKey,
            isSigner: true,
            isWritable: true,
          },
          {
            pubkey: campaign,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: vault,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          }
        ],
        data: instructionData
      });

      const tx = new Transaction().add(donateIx);
      console.log(`${donator.publicKey.toString()} is donating ${amount} sol!`);
    
      await sendAndConfirmTransaction(connection, tx, [donator]);
    }

    const donator1 = new Keypair();
    await airdrop(connection, donator1, 2);
    await donate(campaignPDA, vaultPDA, donator1, 1);

    const donator2 = new Keypair();
    await airdrop(connection, donator2, 2);
    await donate(campaignPDA, vaultPDA, donator2, 1);

    const donator3 = new Keypair();
    await airdrop(connection, donator3, 2);
    await donate(campaignPDA, vaultPDA, donator3, 1);

    let vaultAccountInfo = await connection.getAccountInfo(vaultPDA);
    let vaultBalance = await vaultAccountInfo.lamports / LAMPORTS_PER_SOL;
    /**
     * Assert that vault has received 3 sol
     */
    assert.equal(Math.floor(vaultBalance), 3);

    campaignAccountInfo = await connection.getAccountInfo(campaignPDA);
    campaignState = campaignAccountInfo.data;
    decodedCampaignState = campaignAccountDataLayout
      .decode(campaignState) as CampaignLayout;

    mappedCampaign = mapCampaignLayout(decodedCampaignState);
    /**
     * Assert that amount raised = (1 + 1 + 1) = 3
     */
    assert.equal(mappedCampaign.amountRaised.toNumber(), 3 * LAMPORTS_PER_SOL);

    instruction = 2;
    let withdrawalIx = new TransactionInstruction({
      programId: programId,
      keys: [
        {
          pubkey: fundStarter.publicKey,
          isSigner: true,
          isWritable: true,
        },
        {
          pubkey: campaignPDA,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: vaultPDA,
          isSigner: false,
          isWritable: true
        },
      ],
      data: Buffer.from([instruction])
    });

    let fundStarterInfo = await connection.getAccountInfo(fundStarter.publicKey);
    let initialBalance = fundStarterInfo.lamports;

    let withdrawalTx = new Transaction().add(withdrawalIx);
    console.log("Withdrawing fundraiser earnings");
    await sendAndConfirmTransaction(connection, withdrawalTx, [fundStarter]);

    fundStarterInfo = await connection.getAccountInfo(fundStarter.publicKey);
    let finalBalance = fundStarterInfo.lamports;
    /**
     * assert that fundStarter received 3 sol
     */
    assert.equal(Math.floor((finalBalance - initialBalance)/LAMPORTS_PER_SOL), 3);

    campaignAccountInfo = await connection.getAccountInfo(campaignPDA);
    vaultAccountInfo = await connection.getAccountInfo(vaultPDA);
    /**
     * assert that campaignState and vault accounts are closed
     */
    assert.equal(campaignAccountInfo, null);
    assert.equal(vaultAccountInfo, null);
  });
});

