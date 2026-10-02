<script>
  // The top of the Overview, laid out like a Discord profile: banner, avatar, name, @username
  // with the server tag, badges and profile widgets; package facts and history on the side.
  import Avatar from "./Avatar.svelte";
  import GameIcon from "./GameIcon.svelte";
  import TagChip from "./TagChip.svelte";
  import { allTags, badges, badgeUrl, fmt, monthYear, shortDate, tagHistory } from "../lib/stats.js";

  let { result, games, children } = $props();

  const facts = $derived(result.facts);
  const profile = $derived(facts.profile);
  const sources = $derived(result.sources);
  const packageDate = $derived(sources?.packageDate ?? Date.now());
  const servers = $derived(new Map(facts.servers.map((s) => [s.id, s.name])));

  const avatarUrl = $derived(result.avatar ? URL.createObjectURL(result.avatar) : null);
  // Past avatars, newest first; the current one is often in the folder too, so skip same-size files.
  const pastAvatars = $derived(
    (result.avatars ?? [])
      .filter((a) => a.blob.size !== result.avatar?.size)
      .map((a) => ({ url: URL.createObjectURL(a.blob), ms: a.ms }))
      .sort((a, b) => b.ms - a.ms),
  );

  const history = $derived(tagHistory(allTags(facts, games)));
  // The tag you wear now, with its server's badge as last changed (owners can restyle it later).
  const current = $derived.by(() => {
    const t = history.at(-1);
    const restyled = t && allTags(facts, games).filter((x) => x.kind === "created" && x.guild === t.guild && x.tag === t.tag && x.ms > t.ms).at(-1);
    return restyled ? { ...t, badge: restyled.badge, colours: restyled.colours } : t;
  });
  const tagWhere = (t) => servers.get(t.guild) ?? "a server";
  const tagWhen = (t) => `${shortDate(t.ms)} – ${t.until ? shortDate(t.until) : "now"}`;

  const WIDGETS = { played_games: "Game collection", current_games: "Currently playing", want_to_play_games: "Want to play", favorite_games: "Favorite games" };
  const tagLabel = (t) => t.replace(/_/g, " ").replace(/^./, (c) => c.toUpperCase());
</script>

<section class="profile">
  <div class="banner"></div>
  <div class="body">
    <div class="who">
      <div class="pav"><Avatar src={avatarUrl} id={profile.id} hash={profile.avatar} name={profile.display_name} size={112} /></div>
      <h1>{profile.display_name}</h1>
      <div class="idline">
        <span>@{profile.username}</span>
        {#if current}
          <TagChip tag={current.tag} badge={current.badge} colours={current.colours} tip="{current.tag} · {tagWhere(current)}" sub="Server tag since {shortDate(current.ms)}" />
        {:else if games?.state === "reading"}
          <!-- Which tag you wear is only in the analytics log, read in the background. -->
          <span class="tag-wait" data-tip="Server tag" data-sub="Shows once game history is read"></span>
        {/if}
        <span class="badges" aria-label="Badges">
          {#each badges(profile, packageDate) as b (b.icon)}
            <img src={badgeUrl(b.icon)} alt={b.label} width="22" height="22" data-tip={b.label} />
          {/each}
        </span>
      </div>
      {#each profile.widgets.filter((w) => w.games.length) as w (w.kind)}
        <div class="widget">
          <span>{WIDGETS[w.kind] ?? tagLabel(w.kind)}</span>
          <div class="games">
            {#each w.games as g (g.id)}<GameIcon id={g.id} sub={g.tags.map(tagLabel).join(" · ")} />{/each}
          </div>
        </div>
      {/each}
    </div>

    <div class="side">
      <dl class="facts">
        <div><dt>Member since</dt><dd>{shortDate(profile.created_ms)}</dd></div>
        {#if sources?.packageDate}<div><dt>Package</dt><dd>{monthYear(sources.packageDate)}</dd></div>{/if}
        <div><dt>Friends</dt><dd>{fmt(profile.friends)}{#if profile.blocked}{` · ${fmt(profile.blocked)} blocked`}{/if}</dd></div>
        {#if sources}<div><dt>Servers</dt><dd>{fmt(sources.serversNow)}</dd></div>{/if}
        {#if profile.orbs}<div><dt>Orbs</dt><dd>{fmt(profile.orbs)}</dd></div>{/if}
      </dl>
      {#if history.length > 1}
        <div class="hist">
          <h4>Server tags</h4>
          <div class="chips">
            {#each [...history].reverse() as t (t.ms)}
              <TagChip tag={t.tag} badge={t.badge} colours={t.colours} tip="{t.tag} · {tagWhere(t)}" sub={tagWhen(t)} />
            {/each}
          </div>
        </div>
      {/if}
      {#if pastAvatars.length}
        <div class="hist">
          <h4>Previous avatars</h4>
          <div class="avs">
            {#each pastAvatars as a (a.ms)}
              <img src={a.url} alt="" width="32" height="32" data-tip="Avatar" data-sub="Changed {shortDate(a.ms)}" />
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>
  {@render children?.()}
</section>

<style>
  .profile { background: var(--card); border-radius: var(--radius); overflow: hidden; }
  .banner {
    height: 168px;
    background:
      radial-gradient(120% 140% at 50% 120%, var(--acc) 0%, transparent 55%),
      radial-gradient(60% 90% at 50% 100%, rgba(255, 255, 255, .18) 0%, transparent 60%),
      linear-gradient(#0b023a, #1e0e54);
  }
  .body { padding: 0 24px 24px; display: grid; grid-template-columns: 1fr minmax(240px, auto); gap: 8px 32px; }
  .pav { width: 128px; height: 128px; border-radius: 50%; border: 8px solid var(--card); background: var(--card); margin-top: -64px; }
  h1 { font: 800 26px/1.15 var(--ui); color: var(--t-head); margin: 6px 0 2px; }
  .idline { display: flex; align-items: center; flex-wrap: wrap; gap: 8px; color: var(--t-body); font-size: 15px; }
  .tag-wait { width: 64px; height: 20px; border-radius: 6px; background: var(--raised); animation: wait 1.4s ease-in-out infinite; }
  @keyframes wait { 50% { opacity: .45; } }
  .badges { display: inline-flex; flex-wrap: wrap; gap: 4px; }
  .badges:empty { display: none; }
  .badges img { display: block; width: 22px; height: 22px; }

  .widget {
    display: flex; align-items: center; justify-content: space-between; gap: 16px; margin-top: 14px; max-width: 420px;
    padding: 10px 12px; border-radius: 8px; background: var(--sunk);
  }
  .widget span { font-size: 14px; font-weight: 600; color: var(--t-head); }
  .games { display: flex; gap: 6px; }

  .side { align-self: end; display: grid; gap: 16px; padding-top: 16px; }
  .facts { display: grid; grid-template-columns: auto auto; gap: 14px 32px; margin: 0; }
  .facts dt { font-size: 13px; font-weight: 600; color: var(--t-muted); }
  .facts dd { margin: 2px 0 0; font-size: 14px; color: var(--t-body); }
  .hist h4 { margin: 0 0 6px; font-size: 13px; font-weight: 600; color: var(--t-muted); }
  .chips { display: flex; flex-wrap: wrap; gap: 6px; max-width: 320px; }
  .avs { display: flex; gap: 6px; }
  .avs img { width: 32px; height: 32px; border-radius: 50%; display: block; object-fit: cover; }

  @media (max-width: 760px) {
    .body { grid-template-columns: 1fr; }
    .side { align-self: start; }
  }
  @media (max-width: 620px) {
    .body { padding: 0 16px 16px; }
  }
</style>
