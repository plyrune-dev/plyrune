# Tauten issue submission status

23 November 2026: **neither draft has been posted**.

Both drafts were source-inspected and independently reviewed. Searches found the originating/related issues, but no duplicate covering either proposed correction.

| Draft | Intended title | Current state |
|---|---|---|
| [tauten-stamp-binding.md](tauten-stamp-binding.md) | Bind stamped evidence to its subject as well as its revision | Ready to submit; not filed |
| [tauten-mapping.md](tauten-mapping.md) | Refresh MAPPING.md to reflect shipped evidence, effects and proof capabilities | Ready to submit; not filed |

The GitHub integration returned HTTP 403, “Resource not accessible by integration,” for both create attempts. The browser fallback reached GitHub's sign-in page. A passkey attempt returned “This browser or device is reporting partial passkey support” and “Authentication failed.” No subsequent automatic sign-in attempts or issue writes were made.

There are no issue numbers or created-issue URLs to report. Intended destination: [tauten-dev/tauten](https://github.com/tauten-dev/tauten). The drafts target the existing `later` milestone (4) for owner triage and do not change release scope.

Once authenticated access is available, search once for a newly filed duplicate, submit each exact body, retain the resulting URLs, and update this status. The first draft contains a regression test proposal; it was not compiled in the authoring environment, which lacks rustc.
