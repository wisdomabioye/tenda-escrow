/**
 * Admin instruction suite: initialize_platform + all set_* instructions.
 * Positive paths assert state + events; negative paths assert every
 * on-chain `require` and the protocol-admin gate.
 */
import { BN } from "@coral-xyz/anchor";
import { Keypair, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";

import {
  LIMITS,
  PLATFORM_DEFAULTS,
  TestCtx,
  expectFailure,
  expectTendaError,
  initPlatform,
  newCtx,
  platformArgs,
  sendIxs,
  expectEvent,
} from "./helpers";

describe("admin", () => {
  let ctx: TestCtx;

  beforeEach(() => {
    ctx = newCtx();
  });

  describe("initialize_platform", () => {
    it("initializes platform state and emits PlatformInitialized", async () => {
      const ix = await ctx.program.methods
        .initializePlatform(platformArgs(ctx))
        .accountsPartial({
          platformState: ctx.platformPda,
          payer: ctx.payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .instruction();
      const logs = sendIxs(ctx, [ix], []);

      const state = await ctx.program.account.platformState.fetch(
        ctx.platformPda
      );
      assert.isTrue(state.protocolAdmin.equals(ctx.protocolAdmin.publicKey));
      assert.isTrue(state.disputeAdmin.equals(ctx.disputeAdmin.publicKey));
      assert.isTrue(state.treasury.equals(ctx.treasury.publicKey));
      assert.equal(state.feeBps, PLATFORM_DEFAULTS.feeBps);
      assert.equal(state.seekerFeeBps, PLATFORM_DEFAULTS.seekerFeeBps);
      assert.equal(
        state.approvalWindowSeconds.toNumber(),
        PLATFORM_DEFAULTS.approvalWindowSeconds
      );
      assert.equal(
        state.gracePeriodSeconds.toNumber(),
        PLATFORM_DEFAULTS.gracePeriodSeconds
      );
      assert.equal(state.totalVolume.toNumber(), 0);

      const event = expectEvent(ctx, logs, "platformInitialized");
      assert.equal(event.feeBps as number, PLATFORM_DEFAULTS.feeBps);
    });

    it("rejects fee_bps above the cap", async () => {
      await expectTendaError(
        initPlatform(ctx, { feeBps: LIMITS.maxPlatformFeeBps + 1 }),
        "PlatformFeeTooHigh"
      );
    });

    it("rejects seeker_fee_bps above fee_bps", async () => {
      await expectTendaError(
        initPlatform(ctx, { feeBps: 200, seekerFeeBps: 201 }),
        "SeekerFeeExceedsStandardFee"
      );
    });

    it("rejects approval window below the minimum", async () => {
      await expectTendaError(
        initPlatform(ctx, {
          approvalWindowSeconds: new BN(LIMITS.minApprovalWindowSeconds - 1),
        }),
        "ApprovalWindowOutOfRange"
      );
    });

    it("rejects approval window above the maximum", async () => {
      await expectTendaError(
        initPlatform(ctx, {
          approvalWindowSeconds: new BN(LIMITS.maxApprovalWindowSeconds + 1),
        }),
        "ApprovalWindowOutOfRange"
      );
    });

    it("rejects grace period above the maximum", async () => {
      await expectTendaError(
        initPlatform(ctx, {
          gracePeriodSeconds: new BN(LIMITS.maxGracePeriodSeconds + 1),
        }),
        "GracePeriodOutOfRange"
      );
    });

    it("rejects negative grace period", async () => {
      await expectTendaError(
        initPlatform(ctx, { gracePeriodSeconds: new BN(-1) }),
        "GracePeriodOutOfRange"
      );
    });

    it("rejects double initialization", async () => {
      await initPlatform(ctx);
      await expectFailure(initPlatform(ctx));
    });
  });

  describe("set_* (admin gate + validation)", () => {
    beforeEach(async () => {
      await initPlatform(ctx);
    });

    function adminCall(
      method: "setProtocolAdmin" | "setDisputeAdmin" | "setTreasury",
      newKey: Keypair,
      signer: Keypair
    ) {
      return ctx.program.methods[method](newKey.publicKey)
        .accountsPartial({
          platformState: ctx.platformPda,
          protocolAdmin: signer.publicKey,
        })
        .signers([signer])
        .rpc();
    }

    it("set_treasury updates treasury and emits PlatformConfigChanged", async () => {
      const next = Keypair.generate();
      const ix = await ctx.program.methods
        .setTreasury(next.publicKey)
        .accountsPartial({
          platformState: ctx.platformPda,
          protocolAdmin: ctx.protocolAdmin.publicKey,
        })
        .instruction();
      const logs = sendIxs(ctx, [ix], [ctx.protocolAdmin]);
      expectEvent(ctx, logs, "platformConfigChanged");

      const state = await ctx.program.account.platformState.fetch(
        ctx.platformPda
      );
      assert.isTrue(state.treasury.equals(next.publicKey));
    });

    it("set_dispute_admin rotates the dispute admin", async () => {
      const next = Keypair.generate();
      await adminCall("setDisputeAdmin", next, ctx.protocolAdmin);
      const state = await ctx.program.account.platformState.fetch(
        ctx.platformPda
      );
      assert.isTrue(state.disputeAdmin.equals(next.publicKey));
    });

    it("set_protocol_admin hands over; the old admin is then rejected", async () => {
      const next = Keypair.generate();
      ctx.svm.airdrop(next.publicKey, 1_000_000_000n);
      await adminCall("setProtocolAdmin", next, ctx.protocolAdmin);

      const state = await ctx.program.account.platformState.fetch(
        ctx.platformPda
      );
      assert.isTrue(state.protocolAdmin.equals(next.publicKey));

      await expectTendaError(
        adminCall("setTreasury", Keypair.generate(), ctx.protocolAdmin),
        "NotProtocolAdmin"
      );
      // The new admin works.
      await adminCall("setTreasury", Keypair.generate(), next);
    });

    it("rejects a non-admin signer on every set_*", async () => {
      await expectTendaError(
        adminCall("setTreasury", Keypair.generate(), ctx.outsider),
        "NotProtocolAdmin"
      );
      await expectTendaError(
        ctx.program.methods
          .setFeeBps(100, 50)
          .accountsPartial({
            platformState: ctx.platformPda,
            protocolAdmin: ctx.outsider.publicKey,
          })
          .signers([ctx.outsider])
          .rpc(),
        "NotProtocolAdmin"
      );
    });

    it("set_fee_bps updates both fees; rejects invalid combinations", async () => {
      await ctx.program.methods
        .setFeeBps(300, 150)
        .accountsPartial({
          platformState: ctx.platformPda,
          protocolAdmin: ctx.protocolAdmin.publicKey,
        })
        .signers([ctx.protocolAdmin])
        .rpc();
      const state = await ctx.program.account.platformState.fetch(
        ctx.platformPda
      );
      assert.equal(state.feeBps, 300);
      assert.equal(state.seekerFeeBps, 150);

      await expectTendaError(
        ctx.program.methods
          .setFeeBps(LIMITS.maxPlatformFeeBps + 1, 0)
          .accountsPartial({
            platformState: ctx.platformPda,
            protocolAdmin: ctx.protocolAdmin.publicKey,
          })
          .signers([ctx.protocolAdmin])
          .rpc(),
        "PlatformFeeTooHigh"
      );
      await expectTendaError(
        ctx.program.methods
          .setFeeBps(100, 101)
          .accountsPartial({
            platformState: ctx.platformPda,
            protocolAdmin: ctx.protocolAdmin.publicKey,
          })
          .signers([ctx.protocolAdmin])
          .rpc(),
        "SeekerFeeExceedsStandardFee"
      );
    });

    it("set_approval_window updates; rejects out-of-range", async () => {
      await ctx.program.methods
        .setApprovalWindow(new BN(LIMITS.minApprovalWindowSeconds))
        .accountsPartial({
          platformState: ctx.platformPda,
          protocolAdmin: ctx.protocolAdmin.publicKey,
        })
        .signers([ctx.protocolAdmin])
        .rpc();
      const state = await ctx.program.account.platformState.fetch(
        ctx.platformPda
      );
      assert.equal(
        state.approvalWindowSeconds.toNumber(),
        LIMITS.minApprovalWindowSeconds
      );

      await expectTendaError(
        ctx.program.methods
          .setApprovalWindow(new BN(LIMITS.maxApprovalWindowSeconds + 1))
          .accountsPartial({
            platformState: ctx.platformPda,
            protocolAdmin: ctx.protocolAdmin.publicKey,
          })
          .signers([ctx.protocolAdmin])
          .rpc(),
        "ApprovalWindowOutOfRange"
      );
    });

    it("set_grace_period updates; rejects out-of-range", async () => {
      await ctx.program.methods
        .setGracePeriod(new BN(0))
        .accountsPartial({
          platformState: ctx.platformPda,
          protocolAdmin: ctx.protocolAdmin.publicKey,
        })
        .signers([ctx.protocolAdmin])
        .rpc();
      const state = await ctx.program.account.platformState.fetch(
        ctx.platformPda
      );
      assert.equal(state.gracePeriodSeconds.toNumber(), 0);

      await expectTendaError(
        ctx.program.methods
          .setGracePeriod(new BN(LIMITS.maxGracePeriodSeconds + 1))
          .accountsPartial({
            platformState: ctx.platformPda,
            protocolAdmin: ctx.protocolAdmin.publicKey,
          })
          .signers([ctx.protocolAdmin])
          .rpc(),
        "GracePeriodOutOfRange"
      );
    });
  });
});
