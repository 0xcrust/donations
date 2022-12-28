import {Keypair, LAMPORTS_PER_SOL, Connection, PublicKey} from '@solana/web3.js';
import fs from 'mz/fs';
import path from 'path';
import * as BufferLayout from "buffer-layout";
import BN from 'bn.js';

const PROVIDER_KEYPAIR_PATH = "/home/ademola/.config/solana/id.json";
const PROGRAM_KEYPAIR_PATH = path.resolve(__dirname, '../../program/target/deploy/program-keypair.json');

export function getProviderKeypair(): Keypair {
  try {
    return createKeypairFromFile(PROVIDER_KEYPAIR_PATH);
  } catch(err) {
    console.warn("Failed getting provider keypair");
  }
} 

export function getProgramKeypair(): Keypair {
  try {
    return createKeypairFromFile(PROGRAM_KEYPAIR_PATH);
  } catch(err) {
    console.warn("Failed getting program keypair");
  }
}

export function createKeypairFromFile(
  filePath: string
): Keypair {
  const secretKeyString = fs.readFileSync(filePath, {encoding: 'utf8'});
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  return Keypair.fromSecretKey(secretKey);
}

export async function airdrop(
  connection: Connection, 
  destinationWallet: Keypair, 
  amount: number
) {
  const airdropSignature = await connection.requestAirdrop(destinationWallet.publicKey, 
    amount * LAMPORTS_PER_SOL);

  const latestBlockHash = await connection.getLatestBlockhash();

  await connection.confirmTransaction({
    blockhash: latestBlockHash.blockhash,
    lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
    signature: airdropSignature
  });
}

/**
 * Layouts
 */

// PublicKey
const publicKey = (property = "publicKey") => {
  return BufferLayout.blob(32, property);
}

// u64
const uint64 = (property = "uint64") => {
  return BufferLayout.blob(8, property);
}

// Represents the 200-byte array for the campaign description
const u8Array = (property = "u8ArrayLen200") => {
  return BufferLayout.blob(200, property);
}

export const campaignAccountDataLayout = BufferLayout.struct([
  BufferLayout.u8("isInitialized"),
  publicKey("authority"),
  publicKey("vault"),
  u8Array("description"),
  uint64("target"),
  uint64("amountRaised"),
  BufferLayout.u8("bump"),
]);

export interface CampaignLayout {
  isInitialized: number;
  authority: Uint8Array;
  vault: Uint8Array;
  description: Uint8Array;
  target: Uint8Array;
  amountRaised: Uint8Array;
  bump: number;
}

export interface mappedCampaign {
  isInitialized: boolean;
  authority: PublicKey;
  vault: PublicKey;
  description: string;
  target: BN;
  amountRaised: BN;
  bump: number;
}

export function mapCampaignLayout(layout: CampaignLayout): mappedCampaign {

  const mappedCampaign: mappedCampaign =  {
    isInitialized: Boolean(layout.isInitialized),
    authority: new PublicKey(layout.authority),
    vault: new PublicKey(layout.vault),
    description: layout.description.filter(x => x != 0).toString(),
    target: new BN(layout.target, 'le'),
    amountRaised: new BN(layout.amountRaised, 'le'),
    bump: layout.bump
  }
  
  return mappedCampaign;
}

