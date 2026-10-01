use domain::error::{AppError, AppResult};
use domain::ids::UserId;
use domain::types::org::OrgRole;

/// Checks if a role has at least the required permission level.
pub fn has_minimum_role(user_role: OrgRole, required_role: OrgRole) -> bool {
    let user_level = role_level(user_role);
    let required_level = role_level(required_role);
    user_level >= required_level
}

/// Checks if a user can perform an action on a target user's resources.
pub fn can_manage_user(actor_role: OrgRole, target_role: OrgRole) -> bool {
    // Owners can manage anyone
    if actor_role == OrgRole::Owner {
        return true;
    }
    // Admins can manage members and guests, but not other admins or owners
    if actor_role == OrgRole::Admin {
        return target_role == OrgRole::Member || target_role == OrgRole::Guest;
    }
    // Members and guests cannot manage others
    false
}

/// Checks if a user can manage organization settings.
pub fn can_manage_org(role: OrgRole) -> bool {
    matches!(role, OrgRole::Owner | OrgRole::Admin)
}

/// Checks if a user can delete an organization.
pub fn can_delete_org(role: OrgRole) -> bool {
    role == OrgRole::Owner
}

/// Checks if a user can invite members.
pub fn can_invite_members(role: OrgRole) -> bool {
    matches!(role, OrgRole::Owner | OrgRole::Admin)
}

/// Checks if a user can remove members.
pub fn can_remove_members(actor_role: OrgRole, target_role: OrgRole) -> bool {
    can_manage_user(actor_role, target_role)
}

/// Checks if a user can manage channels.
pub fn can_manage_channels(role: OrgRole) -> bool {
    matches!(role, OrgRole::Owner | OrgRole::Admin)
}

/// Checks if a user can delete any message (not just their own).
pub fn can_delete_any_message(role: OrgRole) -> bool {
    matches!(role, OrgRole::Owner | OrgRole::Admin)
}

/// Checks if a user can pin messages.
pub fn can_pin_messages(role: OrgRole) -> bool {
    matches!(role, OrgRole::Owner | OrgRole::Admin | OrgRole::Member)
}

/// Checks if a user can manage boards.
pub fn can_manage_boards(role: OrgRole) -> bool {
    matches!(role, OrgRole::Owner | OrgRole::Admin | OrgRole::Member)
}

/// Checks if a user can manage tasks.
pub fn can_manage_tasks(role: OrgRole) -> bool {
    matches!(role, OrgRole::Owner | OrgRole::Admin | OrgRole::Member)
}

/// Checks if a user can view audit logs.
pub fn can_view_audit_logs(role: OrgRole) -> bool {
    matches!(role, OrgRole::Owner | OrgRole::Admin)
}

/// Checks if a user can manage webhooks.
pub fn can_manage_webhooks(role: OrgRole) -> bool {
    matches!(role, OrgRole::Owner | OrgRole::Admin)
}

/// Checks if a user can manage bots.
pub fn can_manage_bots(role: OrgRole) -> bool {
    matches!(role, OrgRole::Owner | OrgRole::Admin)
}

/// Returns the numeric level of a role for comparison.
fn role_level(role: OrgRole) -> u8 {
    match role {
        OrgRole::Guest => 0,
        OrgRole::Member => 1,
        OrgRole::Admin => 2,
        OrgRole::Owner => 3,
    }
}

/// Validates that a user has the required role, returning an error if not.
pub fn require_role(user_role: OrgRole, required_role: OrgRole) -> AppResult<()> {
    if has_minimum_role(user_role, required_role) {
        Ok(())
    } else {
        Err(AppError::Authorization(format!(
            "This action requires {:?} role or higher",
            required_role
        )))
    }
}

/// Validates that a user can manage another user.
pub fn require_can_manage_user(
    actor_id: UserId,
    actor_role: OrgRole,
    target_role: OrgRole,
) -> AppResult<()> {
    if actor_id == UserId::new() || can_manage_user(actor_role, target_role) {
        Ok(())
    } else {
        Err(AppError::Authorization(
            "You do not have permission to manage this user".to_string(),
        ))
    }
}
