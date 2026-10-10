/* SPDX-License-Identifier: GPL-3.0-only */
(() => {
  "use strict";
  // Product content and release links work without this enhancement.
  const status = document.getElementById("stats-status");
  if (!status) return;

  const api = "https://api.github.com/repos/qtremors/tremors-music";
  const number = new Intl.NumberFormat();
  const show = (id, value) => {
    const element = document.getElementById(id);
    if (element) element.textContent = number.format(value);
  };
  const request = async (url) => {
    const response = await fetch(url, {
      headers: { Accept: "application/vnd.github+json" },
      signal: AbortSignal.timeout(10000),
    });
    if (!response.ok) throw new Error(`GitHub ${response.status}`);
    return response;
  };
  const assetDownloads = (release) =>
    release.assets.reduce((total, asset) => total + asset.download_count, 0);

  const totalDownloads = async () => {
    let total = 0;
    let url = `${api}/releases?per_page=100`;
    while (url) {
      const response = await request(url);
      const releases = await response.json();
      total += releases.reduce((sum, release) => sum + assetDownloads(release), 0);
      const next = (response.headers.get("link") || "")
        .split(",")
        .find((part) => /;\s*rel="next"/.test(part));
      url = next ? next.match(/<([^>]+)>/)?.[1] : null;
    }
    return total;
  };

  const load = async () => {
    const results = await Promise.allSettled([
      request(api).then((response) => response.json()).then((repo) => {
        show("gh-stars", repo.stargazers_count);
        show("gh-forks", repo.forks_count);
      }),
      request(`${api}/releases/latest`).then((response) => response.json()).then((release) => {
        show("gh-latest-downloads", assetDownloads(release));
        const link = document.createElement("a");
        link.href = release.html_url;
        link.textContent = release.tag_name;
        document.getElementById("release-meta").replaceChildren("Latest on GitHub: ", link);
        document.getElementById("latest-downloads-link").href = release.html_url;
      }),
      totalDownloads().then((total) => show("gh-total-downloads", total)),
    ]);
    status.textContent = results.every((result) => result.status === "fulfilled")
      ? "Live from GitHub. Downloads count release assets."
      : "Some GitHub data is unavailable. Visit GitHub for releases and statistics.";
  };
  load();
})();
