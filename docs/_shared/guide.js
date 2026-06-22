/* Shared copy + run behaviour for macOS Reset guide pages.
   - Every <pre> gets a Run + Copy button pair (top-right).
   - Run posts to /api/run on the local repo server:
       · <pre data-run-id="…"> sends that whitelisted id (always allowed).
       · any other <pre> sends its actual text as a script (same-origin only).
   - Add data-no-run to a <pre> to suppress its Run button (e.g. destructive
     scripts you only ever want to copy).
   - Inline <code> outside a <pre> is click-to-copy. */
(function () {
  async function copyText(text) {
    try {
      await navigator.clipboard.writeText(text);
    } catch (_) {
      const area = document.createElement('textarea');
      area.value = text;
      document.body.appendChild(area);
      area.select();
      try { document.execCommand('copy'); } catch (__) {}
      area.remove();
    }
  }

  function makeCopyButton(pre) {
    const button = document.createElement('button');
    button.className = 'copy-btn';
    button.textContent = 'Copy';
    button.setAttribute('aria-label', 'Copy code to clipboard');
    button.addEventListener('click', async () => {
      await copyText((pre.querySelector('code') || pre).textContent);
      button.textContent = 'Copied!';
      button.classList.add('copied');
      setTimeout(() => {
        button.textContent = 'Copy';
        button.classList.remove('copied');
      }, 1600);
    });
    return button;
  }

  function makeRunButton(pre) {
    const button = document.createElement('button');
    button.className = 'run-btn';
    button.textContent = 'Run';
    button.setAttribute('aria-label', 'Run this command on your Mac');

    const output = document.createElement('div');
    output.className = 'run-output';
    output.hidden = true;
    pre.after(output);

    button.addEventListener('click', async () => {
      const runId = pre.dataset.runId;
      const payload = runId
        ? { id: runId }
        : { script: (pre.querySelector('code') || pre).textContent };

      button.disabled = true;
      button.textContent = 'Running…';
      output.hidden = false;
      output.textContent = 'running…';
      const t0 = performance.now();
      try {
        const res = await fetch('/api/run', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload),
        });
        const data = await res.json();
        if (!res.ok) {
          output.textContent = 'Error: ' + (data.error ?? res.status);
        } else {
          const body = (data.stdout || '') +
            (data.stderr ? '\n[stderr]\n' + data.stderr : '');
          output.textContent = body.trim() || '✓ done (no output)';
          const meta = document.createElement('span');
          meta.className = 'run-meta';
          meta.textContent =
            `$ ${data.command} · exit ${data.exit_code} · ${(performance.now() - t0).toFixed(0)} ms`;
          output.appendChild(meta);
        }
      } catch (e) {
        output.textContent = 'Request failed: ' + e.message +
          '\n(Is the repo server running? Start it with `just serve`.)';
      } finally {
        button.disabled = false;
        button.textContent = 'Run';
      }
    });
    return button;
  }

  // Button group on every code block.
  document.querySelectorAll('pre').forEach(pre => {
    if (pre.querySelector('.pre-btns')) return;

    const group = document.createElement('div');
    group.className = 'pre-btns';

    if (!('noRun' in pre.dataset)) {
      group.appendChild(makeRunButton(pre));
    }
    group.appendChild(makeCopyButton(pre));

    pre.prepend(group);
  });

  // Click-to-copy on inline code.
  document.querySelectorAll('code').forEach(code => {
    if (code.closest('pre')) return;

    code.classList.add('copyable');
    code.title = 'Click to copy';
    code.setAttribute('role', 'button');
    code.setAttribute('tabindex', '0');

    async function copyInline() {
      await copyText(code.textContent);
      code.classList.add('copied');
      const original = code.title;
      code.title = 'Copied!';
      setTimeout(() => {
        code.classList.remove('copied');
        code.title = original;
      }, 1600);
    }

    code.addEventListener('click', copyInline);
    code.addEventListener('keydown', event => {
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        copyInline();
      }
    });
  });
})();
