import * as anchor from '@project-serum/anchor';
import { NftMaker } from '../target/types/nft_maker';

import { Program, BN, IdlAccounts } from "@project-serum/anchor";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { Creator } from "@metaplex-foundation/mpl-token-metadata";
const {
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID, 
  Token
} = require("@solana/spl-token");

//import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";

describe('nft-maker', () => {

  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();

  anchor.setProvider(provider);

  const program = anchor.workspace.NftMaker as Program<NftMaker>;

  const mintKey = Keypair.generate();
  const configKey = Keypair.generate();

  //const recipient = Keypair.generate();

  const recipient = new anchor.web3.PublicKey(
    '2CM9rxUN5CwgYK1GHmUvokWr38LLr7iTcVucSXZW5BZ6',
  );


  const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
    'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
    //'HEWg1Mcwh5bEWUXirSriBucyCGw9wuRzEpioqY4YCZEZ'
  );

  it('Is initialized!', async () => {
    // Add your test here.

    const [vaultkey, nonce] = await PublicKey.findProgramAddress(
      [configKey.publicKey.toBuffer()],
      program.programId
    );

    console.log("mint: ", mintKey.publicKey.toString());
    console.log("configKey: ", configKey.publicKey.toString());
    console.log("vaultkey: ", vaultkey.toString());
  
    const amount = new BN(100000000);
    
    const tx = await program.rpc.initialize(
      nonce,
      provider.wallet.publicKey,
      amount,
      {
        accounts: {
          signer: provider.wallet.publicKey,
          payerVault: vaultkey,
          configuration: configKey.publicKey,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY
        },
        signers: [provider.wallet.payer, configKey],

      });
      
    console.log("tx: ", tx);
  });

  
  it('mint one NFT!', async () => {

    const listener = program.addEventListener("MintEvent", (event, slot) => {
      console.log("slot: ", slot);
      console.log("event status: ", event.status);
      console.log("event mint: ", event.mint);
      console.log("event recipient: ", event.recipient);
      console.log("event nft count: ", event.nftCount);
      program.removeEventListener(listener);
    });

    const assTokenKey = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      mintKey.publicKey,
      recipient
    );

    const [vaultkey, nonce] = await PublicKey.findProgramAddress(
      [configKey.publicKey.toBuffer()],
      program.programId
    );
  
    const [metadatakey, _] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mintKey.publicKey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID,
    );
  
    const [masterkey, __] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mintKey.publicKey.toBuffer(),
        Buffer.from('edition'),
      ],
      TOKEN_METADATA_PROGRAM_ID,
    );

    console.log("mintKey: ", mintKey.publicKey.toString());
    console.log("configKey: ", configKey.publicKey.toString());
    console.log("payerVault: ", vaultkey.toString());

    console.log("recipient: ", recipient.toString());
    console.log("assTokenKey: ", assTokenKey.toString());

    console.log("metadatakey: ", metadatakey.toString());
    console.log("masterkey: ", masterkey.toString());

    const tx = await program.rpc.mintingNft(
      "test",
      "",
      "https://arweave.net/sCuT4ASiUgq7JxgU_3aoq0xJLpwH2Z1z2R2_xwPM8uc",
      1000,
      false,
      {
        accounts: {
          signer: provider.wallet.publicKey,
          recipient: recipient,
          recipientToken: assTokenKey,
          payerVault: vaultkey,
          configuration: configKey.publicKey,
          mint: mintKey.publicKey,
          metadata: metadatakey,
          masteredition: masterkey,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY
        },
        signers: [provider.wallet.payer, mintKey],

      });
      
    console.log("tx:", tx);

  });




});
