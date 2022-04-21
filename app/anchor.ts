import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { MountBreed } from '../target/types/mount_breed';
import { PublicKey, SystemProgram, Transaction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';

const provider = anchor.Provider.env()
anchor.setProvider(provider);

const customTokenB58 = "6ygVeXr1xcnNDxLGPUqUNmBSbVKvixYyEhHusH72kQ5r"
const customMintB58 = "262Nk6611gereoNCt1cdgbKgxFfzLVHMFD3ddQ3Hd3QZ"
const mtmMintB58 = "3xNEV21PVX1xP7o8TCieAJs7XTWwBDDBmbnW4epZxzW9"
const creatorAB58 = "21FefwzZpnpknGeDDb8i6Z5DPGeYr7cZvvgCre7NvPEb"
const creatorBB58 = "21FefwzZpnpknGeDDb8i6Z5DPGeYr7cZvvgCre7NvPEb"
const program = anchor.workspace.MountBreed as Program<MountBreed>;

const main = async () => {
  const [_vault_account_pda, _vault_account_bump] = await PublicKey.findProgramAddress(
    [Buffer.from(anchor.utils.bytes.utf8.encode("token-seed"))],
    program.programId
  );
  const vault_account_pda = _vault_account_pda;
  const vault_account_bump = _vault_account_bump;

  const [_vault_authority_pda, _vault_authority_bump] = await PublicKey.findProgramAddress(
    [Buffer.from(anchor.utils.bytes.utf8.encode("authority-seed"))],
    program.programId
  );
  const vault_authority_pda = _vault_authority_pda;

  const [_escrow_account_pda, _escrow_account_bump] = await PublicKey.findProgramAddress(
    [Buffer.from(anchor.utils.bytes.utf8.encode("escrow-seed"))],
    program.programId
  );
  const escrow_account_bump = _escrow_account_bump
  const escrow_account_pda = _escrow_account_pda

  await program.rpc.genesis(
    vault_account_bump,
    escrow_account_bump,
    {
      accounts: {
        initializer: provider.wallet.publicKey,
        vaultAccount: vault_account_pda,
        mint: new PublicKey(customMintB58),
        initializerDepositTokenAccount: new PublicKey(customTokenB58),
        escrowAccount: escrow_account_pda,
        mintMtm: new PublicKey(mtmMintB58),
        creatorA: new PublicKey(creatorAB58),
        creatorB: new PublicKey(creatorBB58),
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    }
  );
}

// const main = async () => {
//   const [_vault_account_pda, _vault_account_bump] = await PublicKey.findProgramAddress(
//     [Buffer.from(anchor.utils.bytes.utf8.encode("token-seed"))],
//     program.programId
//   );
//   const vault_account_pda = _vault_account_pda;

//   const [_vault_authority_pda, _vault_authority_bump] = await PublicKey.findProgramAddress(
//     [Buffer.from(anchor.utils.bytes.utf8.encode("authority-seed"))],
//     program.programId
//   );
//   const vault_authority_pda = _vault_authority_pda;

//   const [_escrow_account_pda, _escrow_account_bump] = await PublicKey.findProgramAddress(
//     [Buffer.from(anchor.utils.bytes.utf8.encode("escrow-seed"))],
//     program.programId
//   );
//   const escrow_account_pda = _escrow_account_pda

//   // Cancel the escrow.
//   await program.rpc.cancel({
//     accounts: {
//       initializer: provider.wallet.publicKey,
//       initializerDepositTokenAccount: new PublicKey(customTokenB58),
//       vaultAccount: vault_account_pda,
//       vaultAuthority: vault_authority_pda,
//       escrowAccount: escrow_account_pda,
//       tokenProgram: TOKEN_PROGRAM_ID,
//     },
//   });
// }

console.log("Running client...");
main().then(() => console.log("Success"));
