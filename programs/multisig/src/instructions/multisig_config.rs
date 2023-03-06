use anchor_lang::prelude::*;

use crate::errors::*;
use crate::events::ConfigUpdateType;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct MultisigAddMemberArgs {
    new_member: Member,
    /// Memo isn't used for anything, but is included in `AddMemberEvent` that can later be parsed and indexed.
    pub memo: Option<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct MultisigRemoveMemberArgs {
    old_member: Pubkey,
    /// Memo isn't used for anything, but is included in `RemoveMemberEvent` that can later be parsed and indexed.
    pub memo: Option<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct MultisigChangeThresholdArgs {
    new_threshold: u16,
    /// Memo isn't used for anything, but is included in `ChangeThreshold` that can later be parsed and indexed.
    pub memo: Option<String>,
}

#[derive(Accounts)]
pub struct MultisigConfig<'info> {
    #[account(
        mut,
        seeds = [SEED_PREFIX, SEED_MULTISIG, multisig.create_key.as_ref()],
        bump = multisig.bump,
    )]
    multisig: Account<'info, Multisig>,

    /// Multisig `config_authority` that must authorize the configuration change.
    #[account(constraint = config_authority.key() == multisig.config_authority @ MultisigError::Unauthorized)]
    pub config_authority: Signer<'info>,

    /// The account that will be charged in case the multisig account needs to reallocate space,
    /// for example when adding a new member.
    /// This is usually the same as `config_authority`, but can be a different account if needed.
    #[account(mut)]
    pub rent_payer: Signer<'info>,

    /// We might need it in case reallocation is needed.
    pub system_program: Program<'info, System>,
}

impl MultisigConfig<'_> {
    /// Add a member/key to the multisig and reallocate space if necessary.
    pub fn multisig_add_member(ctx: Context<Self>, args: MultisigAddMemberArgs) -> Result<()> {
        let MultisigAddMemberArgs { new_member, memo } = args;

        let system_program = &ctx.accounts.system_program;
        let rent_payer = &ctx.accounts.rent_payer;
        let multisig = &mut ctx.accounts.multisig;
        let multisig_key = multisig.to_account_info().key();

        // Check if we need to reallocate space.
        let reallocated = Multisig::realloc_if_needed(
            multisig.to_account_info(),
            multisig.members.len() + 1,
            rent_payer.to_account_info(),
            system_program.to_account_info(),
        )?;

        if reallocated {
            multisig.reload()?;
        }

        multisig.add_member(new_member);

        multisig.invariant()?;

        multisig.config_updated(
            multisig_key,
            ConfigUpdateType::AddMember { reallocated },
            memo,
        );

        Ok(())
    }

    /// Remove a member/key from the multisig.
    pub fn multisig_remove_member(
        ctx: Context<Self>,
        args: MultisigRemoveMemberArgs,
    ) -> Result<()> {
        let multisig = &mut ctx.accounts.multisig;
        let multisig_key = multisig.to_account_info().key();

        require!(multisig.members.len() > 1, MultisigError::RemoveLastMember);

        multisig.remove_member(args.old_member)?;

        // Update the threshold if necessary.
        if usize::from(multisig.threshold) > multisig.members.len() {
            multisig.threshold = multisig
                .members
                .len()
                .try_into()
                .expect("didn't expect more that `u16::MAX` members");
        };

        multisig.invariant()?;

        multisig.config_updated(multisig_key, ConfigUpdateType::RemoveMember, args.memo);

        Ok(())
    }

    pub fn multisig_change_threshold(
        ctx: Context<Self>,
        args: MultisigChangeThresholdArgs,
    ) -> Result<()> {
        let MultisigChangeThresholdArgs {
            new_threshold,
            memo,
        } = args;

        let multisig = &mut ctx.accounts.multisig;
        let multisig_key = multisig.to_account_info().key();

        multisig.threshold = new_threshold;

        multisig.invariant()?;

        multisig.config_updated(multisig_key, ConfigUpdateType::ChangeThreshold, memo);

        Ok(())
    }
}
