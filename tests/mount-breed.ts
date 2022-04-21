import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { actions, NodeWallet, programs, Wallet } from '@metaplex/js';
import { MountBreed } from '../target/types/mount_breed';
import { PublicKey, SystemProgram, Transaction, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import chai, { assert, expect } from 'chai';
import chaiAsPromised from 'chai-as-promised';
import { createMetadata } from './metaplex';

chai.use(chaiAsPromised);

describe('mount-breed', () => {

  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MountBreed as Program<MountBreed>;

  let mintA = null; // mount 1
  let mintB = null; // mount 2
  let mintC = null; // custom token
  let mintFake = null; // fake mount
  let mintMtm = null; // mtm token
  let mintFakeMtm = null; 

  let userTokenAccountA = null;
  let userTokenAccountB = null;
  let userTokenAccountFake = null;
  let userTokenAccountMtm = null;
  let userTokenAccountFakeMtm = null;
  let initializerTokenAccountC = null;
  let userTokenAccountC = null;

  let mount_a_pda = null;
  let mount_a_bump = null;
  let mount_b_pda = null;
  let mount_b_bump = null;
  let mount_fake_pda = null;
  let mount_fake_bump = null;

  let vault_account_pda = null;
  let vault_account_bump = null;
  let vault_authority_pda = null;
  let escrow_account_pda = null;
  let escrow_account_bump = null;

  let metadataMintA;
  let metadataMintB;
  let metadataMintFake;

  const userMintAAmount = 1;
  const userMintBAmount = 1;
  const userMtmAmount = 400 * LAMPORTS_PER_SOL;
  const initializerMintCAmount = 2202;

  const user = anchor.web3.Keypair.generate();
  const initializer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();
  const creatorA = anchor.web3.Keypair.generate();
  const creatorB = anchor.web3.Keypair.generate();
  const creatorFake = anchor.web3.Keypair.generate();


  it("Initialize mint and token accounts", async () => {
    // airdropping tokens to a initializer
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(initializer.publicKey, 1000000000),
      "confirmed"
    );

    // fund user account
    await provider.send(
      (() => {
        const tx = new Transaction();
        tx.add(
          SystemProgram.transfer({
            fromPubkey: initializer.publicKey,
            toPubkey: user.publicKey,
            lamports: 100000000,
          }),
          SystemProgram.transfer({
            fromPubkey: initializer.publicKey,
            toPubkey: creatorA.publicKey,
            lamports: 100000000,
          }),
          SystemProgram.transfer({
            fromPubkey: initializer.publicKey,
            toPubkey: creatorB.publicKey,
            lamports: 100000000,
          }),
          SystemProgram.transfer({
            fromPubkey: initializer.publicKey,
            toPubkey: creatorFake.publicKey,
            lamports: 100000000,
          }),
        );
        return tx;
      })(),
      [initializer]
    );

    // create mint of mount A & B token
    mintA = await Token.createMint(
      provider.connection,
      initializer,
      creatorA.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    mintB = await Token.createMint(
      provider.connection,
      initializer,
      creatorB.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    mintFake = await Token.createMint(
      provider.connection,
      initializer,
      creatorFake.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    // create mint of custom tokens
    mintC = await Token.createMint(
      provider.connection,
      initializer,
      mintAuthority.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );
    
    // create mint of mtm tokens
    mintMtm = await Token.createMint(
      provider.connection,
      initializer,
      mintAuthority.publicKey,
      null,
      9,
      TOKEN_PROGRAM_ID
    );

    mintFakeMtm = await Token.createMint(
      provider.connection,
      initializer,
      mintAuthority.publicKey,
      null,
      9,
      TOKEN_PROGRAM_ID
    );

    // create user's mount token accounts
    userTokenAccountA = await mintA.createAccount(user.publicKey);
    userTokenAccountB = await mintB.createAccount(user.publicKey);
    userTokenAccountFake = await mintFake.createAccount(user.publicKey);
    userTokenAccountC = await mintC.createAccount(user.publicKey);
    userTokenAccountMtm = await mintMtm.createAccount(user.publicKey);
    userTokenAccountFakeMtm = await mintFakeMtm.createAccount(user.publicKey);

    // create initializer's custom token account
    initializerTokenAccountC = await mintC.createAccount(initializer.publicKey);
    
    // mint to user's mount A/B token account
    await mintA.mintTo(
      userTokenAccountA,
      creatorA.publicKey,
      [creatorA],
      userMintAAmount
    );

    await mintB.mintTo(
      userTokenAccountB,
      creatorB.publicKey,
      [creatorB],
      userMintAAmount
    );

    await mintFake.mintTo(
      userTokenAccountFake,
      creatorFake.publicKey,
      [creatorFake],
      userMintAAmount
    );
    
    // mint to initializer's custom token account
    await mintC.mintTo(
      initializerTokenAccountC,
      mintAuthority.publicKey,
      [mintAuthority],
      initializerMintCAmount
    );
    
    // mint to user's mtm token account
    await mintMtm.mintTo(
      userTokenAccountMtm,
      mintAuthority.publicKey,
      [mintAuthority],
      userMtmAmount
    );
    
    await mintFakeMtm.mintTo(
      userTokenAccountFakeMtm,
      mintAuthority.publicKey,
      [mintAuthority],
      userMtmAmount
    );

    let _userTokenAccountA = await mintA.getAccountInfo(userTokenAccountA);
    let _userTokenAccountB = await mintB.getAccountInfo(userTokenAccountB);
    let _userTokenAccountC = await mintC.getAccountInfo(userTokenAccountC);
    let _userTokenAccountFake = await mintFake.getAccountInfo(userTokenAccountFake);
    let _initializerTokenAccountC = await mintC.getAccountInfo(initializerTokenAccountC);
    let _userTokenAccountMtm = await mintMtm.getAccountInfo(userTokenAccountMtm);
    let _userTokenAccountFakeMtm = await mintFakeMtm.getAccountInfo(userTokenAccountFakeMtm);

    assert.ok(_userTokenAccountA.amount.toNumber() == userMintAAmount);
    assert.ok(_userTokenAccountB.amount.toNumber() == userMintBAmount);
    assert.ok(_userTokenAccountFake.amount.toNumber() == 1);
    assert.ok(_userTokenAccountC.amount.toNumber() == 0);
    assert.ok(_initializerTokenAccountC.amount.toNumber() == initializerMintCAmount);
    assert.ok(_userTokenAccountMtm.amount.toNumber() == userMtmAmount);
    assert.ok(_userTokenAccountFakeMtm.amount.toNumber() == userMtmAmount);
  });

  it("Initialize metadata for mint A, B, Fake", async () => {
    await createMetadata(
      provider.connection,
      new NodeWallet(creatorA),
      mintA.publicKey
    );
    await createMetadata(
      provider.connection,
      new NodeWallet(creatorB),
      mintB.publicKey
    );
    await createMetadata(
      provider.connection,
      new NodeWallet(creatorFake),
      mintFake.publicKey
    );
    metadataMintA = await programs.metadata.Metadata.getPDA(mintA.publicKey);
    metadataMintB = await programs.metadata.Metadata.getPDA(mintB.publicKey);
    metadataMintFake = await programs.metadata.Metadata.getPDA(mintFake.publicKey);
    const metadataAccMintA = await programs.metadata.Metadata.load(
      provider.connection,
      metadataMintA
    );
    const metadataAccMintB = await programs.metadata.Metadata.load(
      provider.connection,
      metadataMintB
    );
    const metadataAccMintFake = await programs.metadata.Metadata.load(
      provider.connection,
      metadataMintFake
    );
    assert.ok(metadataAccMintA.data.data.creators[0].address == creatorA.publicKey.toBase58());
    assert.ok(metadataAccMintB.data.data.creators[0].address == creatorB.publicKey.toBase58());
    assert.ok(metadataAccMintFake.data.data.creators[0].address == creatorFake.publicKey.toBase58());
  });

  it("Genesis transfers token C to vault", async () => {
    const [_vault_account_pda, _vault_account_bump] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("token-seed"))],
      program.programId
    );
    vault_account_pda = _vault_account_pda;
    vault_account_bump = _vault_account_bump;

    const [_vault_authority_pda, _vault_authority_bump] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("authority-seed"))],
      program.programId
    );
    vault_authority_pda = _vault_authority_pda;

    const [_escrow_account_pda, _escrow_account_bump] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("escrow-seed"))],
      program.programId
    );
    escrow_account_bump = _escrow_account_bump
    escrow_account_pda = _escrow_account_pda


    await program.rpc.genesis(
      vault_account_bump,
      escrow_account_bump,
      {
        accounts: {
          initializer: initializer.publicKey,
          vaultAccount: vault_account_pda,
          mint: mintC.publicKey,
          initializerDepositTokenAccount: initializerTokenAccountC,
          escrowAccount: escrow_account_pda,
          mintMtm: mintMtm.publicKey,
          creatorA: creatorA.publicKey,
          creatorB: creatorB.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [initializer],
      }
    );

    let _vault = await mintC.getAccountInfo(vault_account_pda);
    let _initializerTokenAccountC = await mintC.getAccountInfo(initializerTokenAccountC);

    let _escrowAccount = await program.account.escrowAccount.fetch(
      escrow_account_pda
    );

    // Check that initializer's token account C reduced to 0
    assert.ok(_initializerTokenAccountC.amount.toNumber() == 0);

    // Check that the new owner is the PDA, owns 2202 tokens
    assert.ok(_vault.owner.equals(vault_authority_pda));
    assert.ok(_vault.amount.toNumber() == initializerMintCAmount);

    // Check that the values in the escrow account match what we expect.
    assert.ok(_escrowAccount.mtmTokenMint.equals(mintMtm.publicKey));
    assert.ok(_escrowAccount.initializerKey.equals(initializer.publicKey));
    assert.ok(
      _escrowAccount.initializerDepositTokenAccount.equals(initializerTokenAccountC)
    );
    
  });

  it("Initialize data account for mount A", async () => {
    const [_mount_a_pda, _mount_a_bump] = await PublicKey.findProgramAddress(
      [mintA.publicKey.toBytes()],
      program.programId
    );

    mount_a_pda = _mount_a_pda;
    mount_a_bump = _mount_a_bump;

    await program.rpc.initialize(mount_a_bump, {
      accounts: {
        user: user.publicKey,
        mountDataAccount: mount_a_pda,
        mountMintAccount: mintA.publicKey,
        mountTokenAccount: userTokenAccountA,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [user]
    })

    let _mountDataAccount = await program.account.data.fetch(mount_a_pda)

    assert.ok(_mountDataAccount.count == 0)
    assert.ok(_mountDataAccount.bump == mount_a_bump)
    assert.ok(_mountDataAccount.timestamp.toNumber() == 0)
  });

  it("Initialize data account for mount B", async () => {
    const [_mount_b_pda, _mount_b_bump] = await PublicKey.findProgramAddress(
      [mintB.publicKey.toBytes()],
      program.programId
    );

    mount_b_pda = _mount_b_pda;
    mount_b_bump = _mount_b_bump;

    await program.rpc.initialize(mount_b_bump, {
      accounts: {
        user: user.publicKey,
        mountDataAccount: mount_b_pda,
        mountMintAccount: mintB.publicKey,
        mountTokenAccount: userTokenAccountB,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [user]
    })

    let _mountDataAccount = await program.account.data.fetch(mount_b_pda)

    assert.ok(_mountDataAccount.count == 0)
    assert.ok(_mountDataAccount.bump == mount_b_bump)
    assert.ok(_mountDataAccount.timestamp.toNumber() == 0)
  });

  it("Initialize data account for mount Fake", async () => {
    const [_mount_fake_pda, _mount_fake_bump] = await PublicKey.findProgramAddress(
      [mintFake.publicKey.toBytes()],
      program.programId
    );

    mount_fake_pda = _mount_fake_pda;
    mount_fake_bump = _mount_fake_bump;

    await program.rpc.initialize(mount_fake_bump, {
      accounts: {
        user: user.publicKey,
        mountDataAccount: mount_fake_pda,
        mountMintAccount: mintFake.publicKey,
        mountTokenAccount: userTokenAccountFake,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [user]
    })

    let _mountDataAccount = await program.account.data.fetch(mount_fake_pda)

    assert.ok(_mountDataAccount.count == 0)
    assert.ok(_mountDataAccount.bump == mount_fake_bump)
    assert.ok(_mountDataAccount.timestamp.toNumber() == 0)
  });

  it("Shoud fail to redeem custom token using fake mtm tokens", async () => {
    await expect(
      program.rpc.redeem({
        accounts: {
          user: user.publicKey,
          mountDataAccountA: mount_a_pda,
          mountDataAccountB: mount_b_pda,
          userMountMintAccountA: mintA.publicKey,
          userMountTokenAccountA: userTokenAccountA,
          metadataMountA: metadataMintA,
          userMountMintAccountB: mintB.publicKey,
          userMountTokenAccountB: userTokenAccountB,
          metadataMountB: metadataMintB,
          userCustomTokenAccount: userTokenAccountC,
          userMtmTokenAccount: userTokenAccountFakeMtm,
          vaultAccount: vault_account_pda,
          escrowAccount: escrow_account_pda,
          mintMtm: mintMtm.publicKey,
          vaultAuthority: vault_authority_pda,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers : [user]
      })
    ).to.be.rejectedWith(
      '2003: A raw constraint was violated'
    );
  });

  it("Shoud fail to redeem token passing wrong metadata account", async () => {
    await expect(
      program.rpc.redeem({
        accounts: {
          user: user.publicKey,
          mountDataAccountA: mount_a_pda,
          mountDataAccountB: mount_b_pda,
          userMountMintAccountA: mintA.publicKey,
          userMountTokenAccountA: userTokenAccountA,
          metadataMountA: metadataMintB,
          userMountMintAccountB: mintB.publicKey,
          userMountTokenAccountB: userTokenAccountB,
          metadataMountB: metadataMintB,
          userCustomTokenAccount: userTokenAccountC,
          userMtmTokenAccount: userTokenAccountMtm,
          vaultAccount: vault_account_pda,
          escrowAccount: escrow_account_pda,
          mintMtm: mintMtm.publicKey,
          vaultAuthority: vault_authority_pda,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers : [user]
      })
    ).to.be.rejectedWith(
      'failed to send transaction: Transaction simulation failed: Error processing Instruction 0: Program failed to complete'
    );
  });


  it("Fails to redeem passing fake mount", async () => {
    await expect(
      program.rpc.redeem({
        accounts: {
          user: user.publicKey,
          mountDataAccountA: mount_fake_pda,
          mountDataAccountB: mount_b_pda,
          userMountMintAccountA: mintFake.publicKey,
          userMountTokenAccountA: userTokenAccountFake,
          metadataMountA: metadataMintFake,
          userMountMintAccountB: mintB.publicKey,
          userMountTokenAccountB: userTokenAccountB,
          metadataMountB: metadataMintB,
          userCustomTokenAccount: userTokenAccountC,
          userMtmTokenAccount: userTokenAccountMtm,
          vaultAccount: vault_account_pda,
          escrowAccount: escrow_account_pda,
          mintMtm: mintMtm.publicKey,
          vaultAuthority: vault_authority_pda,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers : [user]
      })
    ).to.be.rejectedWith(
      'failed to send transaction: Transaction simulation failed: Error processing Instruction 0: custom program error: 0x1'
    );
  });


  it("Redeems custom token", async () => {
    await program.rpc.redeem({
      accounts: {
        user: user.publicKey,
        mountDataAccountA: mount_a_pda,
        mountDataAccountB: mount_b_pda,
        userMountMintAccountA: mintA.publicKey,
        userMountTokenAccountA: userTokenAccountA,
        metadataMountA: metadataMintA,
        userMountMintAccountB: mintB.publicKey,
        userMountTokenAccountB: userTokenAccountB,
        metadataMountB: metadataMintB,
        userCustomTokenAccount: userTokenAccountC,
        userMtmTokenAccount: userTokenAccountMtm,
        vaultAccount: vault_account_pda,
        escrowAccount: escrow_account_pda,
        mintMtm: mintMtm.publicKey,
        vaultAuthority: vault_authority_pda,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers : [user]
    })

    let _mountDataAccountA = await program.account.data.fetch(mount_a_pda);
    let _mountDataAccountB = await program.account.data.fetch(mount_b_pda);
    let _userTokenAccountC = await mintC.getAccountInfo(userTokenAccountC);
    let _userTokenAccountMtm = await mintMtm.getAccountInfo(userTokenAccountMtm);

    assert.ok(_mountDataAccountA.count == 1);
    assert.ok(_mountDataAccountA.timestamp.toNumber() > 0);
    assert.ok(_mountDataAccountB.count == 1);
    assert.ok(_mountDataAccountB.timestamp.toNumber() > 0);
    assert.ok(_userTokenAccountC.amount.toNumber() == 1);
    assert.ok(_userTokenAccountMtm.amount.toNumber() == userMtmAmount - (200 * LAMPORTS_PER_SOL));
    
  });

  it("Shoud fail to redeem custom token before 7 days buffer", async () => {
    await expect(
      program.rpc.redeem({
        accounts: {
          user: user.publicKey,
          mountDataAccountA: mount_a_pda,
          mountDataAccountB: mount_b_pda,
          userMountMintAccountA: mintA.publicKey,
          userMountTokenAccountA: userTokenAccountA,
          metadataMountA: mintA.publicKey,
          userMountMintAccountB: mintB.publicKey,
          userMountTokenAccountB: userTokenAccountB,
          metadataMountB: mintB.publicKey,
          userCustomTokenAccount: userTokenAccountC,
          userMtmTokenAccount: userTokenAccountMtm,
          vaultAccount: vault_account_pda,
          escrowAccount: escrow_account_pda,
          mintMtm: mintMtm.publicKey,
          vaultAuthority: vault_authority_pda,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers : [user]
      })
    ).to.be.rejectedWith(
      '2003: A raw constraint was violated'
    );
  });

  it("Cancel returns token C to initializer", async () => {
    // Check that initializer's token account C reduced to 0
    let _beforeInitializerTokenAccountC = await mintC.getAccountInfo(initializerTokenAccountC);
    assert.ok(_beforeInitializerTokenAccountC.amount.toNumber() == 0);

    // Cancel the escrow.
    await program.rpc.cancel({
      accounts: {
        initializer: initializer.publicKey,
        initializerDepositTokenAccount: initializerTokenAccountC,
        vaultAccount: vault_account_pda,
        vaultAuthority: vault_authority_pda,
        escrowAccount: escrow_account_pda,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [initializer]
    });

    
    let _initializerTokenAccountC = await mintC.getAccountInfo(initializerTokenAccountC);
    // Check that initializer's token account C has been returned
    assert.ok(_initializerTokenAccountC.amount.toNumber() == 2201);
    // Check that owner of initializerTokenAccountC belongs to initializer
    assert.ok(_initializerTokenAccountC.owner.equals(initializer.publicKey));

    // Check vault account has been closed
    await expect(mintC.getAccountInfo(vault_account_pda)).to.be.rejectedWith(
      'Failed to find account'
    );

  });
});