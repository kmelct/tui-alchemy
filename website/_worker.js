const CANONICAL_HOST = "tui-alchemy.sh";
const INSTALLER_HOST = "i.tui-alchemy.sh";
const WWW_HOST = "www.tui-alchemy.sh";
const PAGES_HOST = "tui-alchemy.pages.dev";
const PAGES_SUFFIX = ".tui-alchemy.pages.dev";

function redirectToCanonical(url) {
  url.hostname = CANONICAL_HOST;
  url.protocol = "https:";
  return Response.redirect(url.toString(), 301);
}

function isPagesHost(hostname) {
  return hostname === PAGES_HOST || hostname.endsWith(PAGES_SUFFIX);
}
function assetRequest(request, pathname) {
  const url = new URL(request.url);
  url.pathname = pathname;
  url.search = "";
  return new Request(url, request);
}

async function serveAsset(request, env, pathname, contentType) {
  const response = await env.ASSETS.fetch(assetRequest(request, pathname));
  if (!response.ok) return response;

  const headers = new Headers(response.headers);
  headers.set("content-type", contentType);
  headers.set("cache-control", "public, max-age=300");
  return new Response(response.body, { status: response.status, headers });
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    if (url.hostname === WWW_HOST || isPagesHost(url.hostname)) {
      return redirectToCanonical(url);
    }

    if (url.hostname === INSTALLER_HOST) {
      if (url.pathname === "/" || url.pathname === "") {
        return serveAsset(request, env, "/i.tui-alchemy.sh", "text/x-shellscript; charset=utf-8");
      }
      if (url.pathname === "/install.ps1") {
        return serveAsset(request, env, "/install.ps1", "text/plain; charset=utf-8");
      }
    }

    return env.ASSETS.fetch(request);
  },
};
