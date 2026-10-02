// The Statistics page: how often Discord logged you doing things, from the event counts of the
// activity logs. The logs overlap (the trust & safety and analytics logs repeat many reporting
// events), so each event type counts once, at its highest count in any log.

// [label, event types to add up]
const GROUPS = [
  {
    title: "Messages", icon: "chat", items: [
      ["Messages sent", ["send_message"]],
      ["Messages edited", ["message_edited"]],
      ["Messages deleted", ["message_deleted"]],
      ["Replies", ["reply_message_started"]],
      ["Messages pinned", ["pin_message"]],
      ["Reactions added", ["add_reaction"]],
      ["Reactions removed", ["remove_reaction"]],
      ["Files uploaded", ["attachment_upload_finished"]],
      ["GIFs sent", ["message_sent_with_gif"]],
      ["Stickers sent", ["sticker_attached"]],
      ["Message links copied", ["message_link_copied"]],
      ["Commands used", ["application_command_used"]],
    ],
  },
  {
    title: "Voice and calls", icon: "voice", items: [
      ["Voice channel joins", ["join_voice_channel"]],
      ["Calls started", ["start_call"]],
      ["Calls joined", ["join_call"]],
      ["Calls rung", ["ring_call"]],
      ["Screen shares", ["screenshare_finished"]],
      ["Times muted or unmuted", ["input_mute_toggled"]],
      ["Times deafened or undeafened", ["self_deafen_toggled"]],
      ["Voice effects sent", ["voice_channel_effect_sent"]],
    ],
  },
  {
    title: "Servers", icon: "servers", items: [
      ["Servers opened", ["guild_viewed"]],
      ["Server joins", ["guild_joined"]],
      ["Server leaves", ["leave_guild"]],
      ["Servers created", ["create_guild"]],
      ["Servers deleted", ["delete_guild"]],
      ["Channels created", ["create_channel"]],
      ["Channels deleted", ["channel_deleted"]],
      ["Channel edits", ["channel_updated"]],
      ["Role edits", ["guild_role_updated"]],
      ["Server settings changes", ["guild_settings_updated"]],
      ["Moderation actions", ["moderation_action"]],
      ["Invites created", ["create_instant_invite"]],
      ["Invites copied", ["copy_instant_invite"]],
      ["Invites sent", ["invite_sent"]],
      ["Invites accepted", ["accepted_instant_invite"]],
      ["Bots added", ["guild_bot_added"]],
      ["Webhooks created", ["webhook_created"]],
      ["Emoji uploaded", ["create_emoji"]],
      ["Emoji deleted", ["delete_emoji"]],
      ["Stickers uploaded", ["create_sticker"]],
      ["Sounds uploaded", ["soundboard_sound_uploaded"]],
      ["Threads started", ["thread_creation_started"]],
      ["Member lists opened", ["member_list_viewed"]],
      ["Times viewed as a role", ["view_as_roles_selected"]],
    ],
  },
  {
    title: "People", icon: "people", items: [
      ["Friends list opened", ["friends_list_viewed"]],
      ["DM list opened", ["dm_list_viewed"]],
      ["Friend suggestions skipped", ["friend_suggestion_skipped"]],
      ["Message requests answered", ["message_request_action"]],
      ["People added to group chats", ["add_channel_recipient"]],
      ["People removed from group chats", ["remove_channel_recipient"]],
      ["People blocked", ["block_user_confirmed"]],
    ],
  },
  {
    title: "The app", icon: "desktop", items: [
      ["Times opened", ["app_opened"]],
      ["Times sent to the background", ["app_background"]],
      ["Crashes", ["app_native_crash", "app_crashed"]],
      ["Searches", ["search_started"]],
      ["Keyboard shortcuts", ["keyboard_shortcut_used"]],
      ["Keyboard mode toggles", ["keyboard_mode_toggled"]],
      ["Links clicked", ["link_clicked"]],
      ["Notifications clicked", ["notification_clicked"]],
      ["Notification settings changes", ["notification_settings_updated"]],
      ["Popouts opened", ["open_popout"]],
      ["Pop-up windows opened", ["open_modal"]],
      ["Changelogs opened", ["change_log_opened"]],
      ["Nitro ads seen", ["premium_upsell_viewed"]],
    ],
  },
  {
    title: "Your account", icon: "shield", items: [
      ["Logins", ["login_successful"]],
      ["Login attempts", ["login_attempted"]],
      ["Captchas shown", ["captcha_served"]],
      ["Settings changes", ["update_user_settings"]],
      ["Custom status changes", ["custom_status_updated"]],
      ["Avatar changes", ["user_avatar_updated"]],
      ["Apps authorized", ["oauth2_authorize_accepted"]],
      ["Developer apps created", ["application_created"]],
      ["Data package requests", ["data_request_initiated"]],
      ["Emails from Discord", ["email_sent"]],
    ],
  },
  {
    title: "Shopping", icon: "coin", items: [
      ["Checkouts started", ["payment_flow_started"]],
      ["Checkouts cancelled", ["payment_flow_canceled"]],
      ["Gift codes created", ["gift_code_created"]],
      ["Gift links copied", ["gift_code_copied"]],
    ],
  },
];

/** Every event type's count, the highest across the logs that were read. */
export function eventCounts(result, games) {
  const counts = new Map();
  const add = (types) => {
    for (const [t, n] of types ?? []) counts.set(t, Math.max(counts.get(t) ?? 0, n));
  };
  for (const f of Object.values(result.sources?.folders ?? {})) add(f.types);
  add(games?.types);
  return counts;
}

/** The groups of the Statistics page with their non-zero counts: [{ title, icon, items: [{ label, n }] }]. */
export function statGroups(result, games) {
  const counts = eventCounts(result, games);
  return GROUPS.map((g) => ({
    ...g,
    items: g.items.map(([label, types]) => ({ label, n: types.reduce((s, t) => s + (counts.get(t) ?? 0), 0) })).filter((x) => x.n > 0),
  })).filter((g) => g.items.length);
}
