<script module>
  // Stays revealed while the page is open, so switching tabs doesn't hide it again.
  let shownOnce = false;
</script>

<script>
  import Avatar from "../components/Avatar.svelte";
  import { appInfo } from "../lib/discord.js";
  import { countryName, flag } from "../lib/names.js";
  import { fmt, fmtCompact, monthYear, shortDate } from "../lib/stats.js";

  let { result, view, games } = $props();

  let revealed = $state(shownOnce);
  const reveal = () => { revealed = shownOnce = true; };

  const facts = $derived(result.facts);
  const a = $derived(facts.account);
  const ad = $derived(facts.ad_profile);
  const d = $derived(facts.devices);
  const folders = $derived(result.sources?.folders ?? {});

  const yesNo = (v) => (v == null ? "Not stored" : v ? "Yes" : "No");
  const strip = (s) => s.replace(/^[^>]*>\s*/, "");
  const audience = (s) => s.replace(/^mobile_install_/, "Installed ").replace(/_/g, " ");

  let gamesKnown = $state({});
  $effect(() => {
    if (!revealed) return;
    for (const id of (ad?.game_ids_l730 ?? []).slice(0, 14)) {
      appInfo(id).then((x) => x && (gamesKnown = { ...gamesKnown, [id]: x }));
    }
  });

  const PRIVACY = {
    detectPlatformAccounts: "Suggest connecting accounts it detects",
    contactSyncEnabled: "Sync phone contacts",
    passwordless: "Passwordless login",
    allowAccessibilityDetection: "Let Discord detect accessibility tools",
    allowActivityPartyPrivacyFriends: "Friends can join your activities",
    allowActivityPartyPrivacyVoiceChannel: "Voice channel members can join your activities",
  };
  const privacy = $derived(Object.entries(a?.privacy ?? {}).filter(([k, v]) => PRIVACY[k] && typeof v !== "object"));

  const typesTop = $derived.by(() => {
    const all = new Map();
    for (const list of [folders.reporting?.types, folders.tns?.types, games?.types]) {
      for (const [t, n] of list ?? []) all.set(t, (all.get(t) ?? 0) + n);
    }
    return [...all].sort((x, y) => y[1] - x[1]);
  });
  const gb = (n) => `${(n / 2 ** 30).toFixed(1)} GB`;
  const FOLDERS = { reporting: "Reporting", tns: "Trust & safety", analytics: "Analytics", modeling: "Modeling" };
</script>

<div class="page">
  {#if !revealed}
    <section class="card gate">
      <h2>What Discord knows</h2>
      <p>
        Personal details from your package: your email, phone number, IP addresses, the places you've used Discord,
        your notes on people and Discord's advertising profile of you.
      </p>
      <button class="cta" onclick={reveal}>Show my details</button>
    </section>
  {:else}
    <div class="grid12">
      {#if a}
        <section class="card s6">
          <h2>Account</h2>
          <dl class="kv">
            <div><dt>Email</dt><dd>{a.email ?? "Not stored"}</dd></div>
            <div><dt>Phone</dt><dd>{a.phone ?? "Not stored"}</dd></div>
            <div><dt>Last IP address</dt><dd class="mono">{a.ip ?? "Not stored"}</dd></div>
            <div><dt>Email verified</dt><dd>{yesNo(a.verified)}</dd></div>
            <div><dt>Date of birth</dt><dd>{a.date_of_birth ?? "Not stored"}</dd></div>
            <div><dt>Age check</dt><dd>{a.age_assurance?.inferred_age_group ? `Inferred: ${a.age_assurance.inferred_age_group}` : a.age_assurance?.method ?? "None"}</dd></div>
            <div><dt>Predicted age and gender</dt><dd>{a.predicted_age ?? "None"} · {a.predicted_gender ?? "None"}</dd></div>
            <div><dt>Temporary ban</dt><dd>{a.temp_banned_until ? shortDate(Date.parse(a.temp_banned_until)) : "None"}</dd></div>
          </dl>
        </section>
      {/if}

      {#if ad}
        <section class="card s6">
          <h2>Your ad profile</h2>
          <dl class="kv">
            {#if ad.age_group}<div><dt>Age group</dt><dd>{ad.age_group}</dd></div>{/if}
            {#if ad.reg_country_code}<div><dt>Registered in</dt><dd>{flag(ad.reg_country_code)} {countryName(ad.reg_country_code)}{ad.reg_region ? ` · ${ad.reg_region}` : ""}</dd></div>{/if}
            {#if ad.primary_platform_l30}<div><dt>Main platform, last 30 days</dt><dd>{ad.primary_platform_l30}</dd></div>{/if}
            {#if ad.has_active_subscription != null}<div><dt>Paying subscriber</dt><dd>{yesNo(ad.has_active_subscription)}</dd></div>{/if}
            {#if ad.maid_state}<div><dt>Mobile advertising ID</dt><dd>{ad.maid_state.replace(/_/g, " ")}</dd></div>{/if}
          </dl>
          {#each [["Themes", ad.theme_names_l90], ["Mobile games", ad.mobile_genre_names], ["Movies and TV", ad.movie_and_tv_genre_names], ["Music and audio", ad.music_and_audio_genre_names]] as [label, list]}
            {#if list?.length}
              <h3>{label}</h3>
              <div class="chips">{#each list as t}<span>{strip(t).toLowerCase().replace(/^./, (c) => c.toUpperCase())}</span>{/each}</div>
            {/if}
          {/each}
          {#if ad.custom_audiences?.length}
            {@const named = ad.custom_audiences.filter((x) => !/^\d+$/.test(x))}
            <h3>Ad audiences you're in</h3>
            <div class="chips">
              {#each named as t}<span>{audience(t)}</span>{/each}
              {#if ad.custom_audiences.length > named.length}<span class="muted">+{ad.custom_audiences.length - named.length} unnamed</span>{/if}
            </div>
          {/if}
          {#if ad.game_ids_l730?.length}
            <h3>Games it thinks you play</h3>
            <div class="chips">
              {#each [...new Set(ad.game_ids_l730)].slice(0, 14) as id (id)}
                <span class="game">{#if gamesKnown[id]?.icon}<img src={gamesKnown[id].icon} alt="" width="18" height="18" />{/if}{gamesKnown[id]?.name ?? "…"}</span>
              {/each}
            </div>
          {/if}
        </section>
      {/if}

      {#if d}
        <section class="card s6">
          <h2>Countries <small>(from your IP address)</small></h2>
          <ol class="places">
            {#each d.countries.slice(0, 12) as c, i (i)}
              <li>
                <span class="flag" aria-hidden="true">{flag(c.name)}</span>
                <div><strong>{countryName(c.name)}</strong><small>{monthYear(c.first_ms)} – {monthYear(c.last_ms)}</small></div>
                <span class="n num">{fmt(c.count)}</span>
              </li>
            {/each}
          </ol>
        </section>

        <section class="card s6">
          <h2>Cities and internet providers</h2>
          <ol class="places">
            {#each d.cities.slice(0, 8) as c, i (i)}
              {@const [city, cc] = c.name.split(/, (?=[A-Z]{2}$)/)}
              <li>
                <span class="flag" aria-hidden="true">{flag(cc ?? "")}</span>
                <div><strong>{city}</strong><small>{monthYear(c.first_ms)} – {monthYear(c.last_ms)}</small></div>
                <span class="n num">{fmt(c.count)}</span>
              </li>
            {/each}
          </ol>
          {#if d.isps.length}
            <h3>Providers</h3>
            <div class="chips">{#each d.isps.slice(0, 10) as x, i (i)}<span>{x.name}</span>{/each}</div>
          {/if}
        </section>
      {/if}

      {#if a?.sessions.length}
        <section class="card s12">
          <h2>Logged-in sessions</h2>
          <table>
            <thead><tr><th>Where</th><th>IP address</th><th>Logged in</th><th>Last used</th><th>2FA</th></tr></thead>
            <tbody>
              {#each a.sessions.slice(0, 12) as x, i (i)}
                <tr>
                  <td>{[x.platform, x.os].filter(Boolean).join(" on ") || "Unknown"}</td>
                  <td class="mono">{x.ip ?? "–"}</td>
                  <td>{shortDate(x.created_ms)}</td>
                  <td>{x.last_used_ms ? shortDate(x.last_used_ms) : "–"}</td>
                  <td>{x.mfa ? "Yes" : "No"}</td>
                </tr>
              {/each}
            </tbody>
          </table>
          {#if a.sessions.length > 12}<p class="more">And {a.sessions.length - 12} more.</p>{/if}
        </section>
      {/if}

      {#if a?.connections.length}
        <section class="card s6">
          <h2>Connected accounts</h2>
          <ol class="conns">
            {#each a.connections as c, i (i)}
              <li><b>{c.kind}</b><span>{c.name}</span><small>{c.visible ? "On profile" : "Hidden"}</small></li>
            {/each}
          </ol>
        </section>
      {/if}

      {#if a}
        <section class="card s6">
          <h2>Settings</h2>
          <dl class="kv">
            <div><dt>Muted servers</dt><dd>{fmt(a.muted_servers)} of {fmt(a.server_settings)}</dd></div>
            {#each privacy as [k, v] (k)}<div><dt>{PRIVACY[k]}</dt><dd>{yesNo(v)}</dd></div>{/each}
          </dl>
        </section>
      {/if}

      {#if a?.notes.length}
        <section class="card s12">
          <h2>Notes you wrote on people</h2>
          <ol class="notes">
            {#each a.notes as [id, note] (id)}
              {@const p = view.people.get(id)}
              <li>
                <Avatar {id} hash={p?.avatar} name={p?.display_name ?? "?"} size={32} />
                <div><strong>{p?.display_name ?? "Someone"}</strong><p>{note}</p></div>
              </li>
            {/each}
          </ol>
        </section>
      {/if}

      <section class="card s12">
        <h2>How much Discord logged{#if facts.data_requests.length} <small>({facts.data_requests.length} data requests since {monthYear(facts.data_requests[0])})</small>{/if}</h2>
        <dl class="folders">
          {#each Object.entries(FOLDERS) as [k, label] (k)}
            {@const f = k === "analytics" && games?.events ? { ...folders[k], events: games.events, read: true } : folders[k]}
            <div>
              <dt>{label}</dt>
              <dd>{f ? gb(f.size) : "Not in package"}</dd>
              <small>{f?.read ? `${fmtCompact(f.events)} events` : f ? "Not read" : ""}</small>
            </div>
          {/each}
        </dl>
        {#if typesTop.length}
          <h3>Most logged events</h3>
          <ol class="types">
            {#each typesTop.slice(0, 20) as [t, n], i (i)}
              <li><code>{t}</code><span class="meter"><i style:width="{(n / typesTop[0][1]) * 100}%"></i></span><small class="num">{fmtCompact(n)}</small></li>
            {/each}
          </ol>
          <p class="more">{fmt(typesTop.length)} different kinds of events in the folders read.</p>
        {/if}
      </section>
    </div>
  {/if}
</div>

<style>
  .gate { max-width: 640px; margin: 48px auto; text-align: center; padding: 32px; }
  .gate p { color: var(--t-muted); margin: 0 auto 20px; max-width: 52ch; }
  .cta { border: 0; border-radius: 6px; padding: 10px 18px; font-weight: 600; cursor: pointer; background: var(--acc-strong); color: var(--on-acc); }

  .kv { display: grid; gap: 10px; margin: 0; }
  .kv div { display: grid; grid-template-columns: minmax(140px, 0.9fr) 1.1fr; gap: 12px; align-items: baseline; }
  .kv dt { color: var(--t-muted); font-size: 14px; }
  .kv dd { margin: 0; color: var(--t-head); font-weight: 600; overflow-wrap: anywhere; }
  .mono, code { font-family: var(--mono); font-size: 13px; }

  h3 { margin: 18px 0 8px; font-size: 13px; font-weight: 600; color: var(--t-muted); }
  .chips { display: flex; flex-wrap: wrap; gap: 6px; }
  .chips span { font-size: 13px; padding: 3px 8px; border-radius: 6px; background: var(--sunk); color: var(--t-body); display: inline-flex; align-items: center; gap: 6px; }
  .chips img { border-radius: 4px; }

  .places { list-style: none; margin: 0; padding: 0; display: grid; gap: 10px; }
  .places li { display: grid; grid-template-columns: 28px 1fr auto; align-items: center; gap: 10px; }
  .flag { font-size: 22px; line-height: 1; }
  .places strong { display: block; color: var(--t-head); font-weight: 600; }
  .places small { color: var(--t-muted); font-size: 12px; }
  .n { color: var(--t-muted); font-size: 13px; }

  table { width: 100%; border-collapse: collapse; font-size: 14px; }
  th { text-align: left; font-size: 12px; font-weight: 600; color: var(--t-muted); padding: 0 8px 8px; border-bottom: 1px solid var(--line); }
  td { padding: 8px; border-bottom: 1px solid var(--sunk); color: var(--t-body); }
  .more { color: var(--t-muted); font-size: 13px; margin: 10px 0 0; }

  .conns { list-style: none; margin: 0; padding: 0; display: grid; gap: 8px; }
  .conns li { display: grid; grid-template-columns: 90px 1fr auto; gap: 10px; align-items: baseline; font-size: 14px; }
  .conns b { color: var(--t-muted); font-weight: 600; text-transform: capitalize; }
  .conns span { color: var(--t-head); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .conns small { color: var(--t-faint); font-size: 12px; }

  .notes { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 10px; }
  .notes li { display: grid; grid-template-columns: 32px 1fr; gap: 10px; padding: 10px 12px; border-radius: 8px; background: var(--sunk); }
  .notes strong { color: var(--t-head); font-weight: 600; }
  .notes p { margin: 2px 0 0; color: var(--t-body); font-size: 14px; white-space: pre-wrap; overflow-wrap: anywhere; }

  .folders { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin: 0; }
  .folders div { padding: 12px 14px; border-radius: 8px; background: var(--sunk); }
  .folders dt { color: var(--t-muted); font-size: 13px; font-weight: 600; }
  .folders dd { margin: 4px 0 0; font: 900 20px var(--display); color: var(--t-head); }
  .folders small { color: var(--t-muted); font-size: 12px; }
  .types { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(2, 1fr); gap: 8px 28px; }
  .types li { display: grid; grid-template-columns: minmax(0, 1.2fr) 1fr 56px; align-items: center; gap: 10px; }
  .types code { color: var(--t-head); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .types small { color: var(--t-muted); font-size: 12px; text-align: right; }

  @media (max-width: 760px) {
    .folders { grid-template-columns: repeat(2, 1fr); }
    .types { grid-template-columns: 1fr; }
    .kv div { grid-template-columns: 1fr; gap: 2px; }
  }
</style>
