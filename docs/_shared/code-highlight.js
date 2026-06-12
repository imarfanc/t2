(function () {
  const LANGS = {
    bash: [
      { re: /#[^\n]*/g, cls: 'cmt' },
      { re: /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/g, cls: 'str' },
      { re: /\$\{?[A-Za-z_][A-Za-z0-9_]*\}?|\$\d+/g, cls: 'var' },
      { re: /--?[A-Za-z][\w-]*/g, cls: 'flag' },
      { re: /\b(?:curl|source|cd|mkdir|chmod|rustc|cargo|exec|echo|test|dirname|pwd|sh|bin)\b/g, cls: 'kw' },
      { re: /[|;&<>]{1,2}|\|\|/g, cls: 'op' },
    ],
    rust: [
      { re: /\/\/[^\n]*/g, cls: 'cmt' },
      { re: /"(?:\\.|[^"\\])*"/g, cls: 'str' },
      { re: /\b(?:fn|let|mut|pub|use|mod|struct|enum|impl|match|if|else|for|while|return|Self|self|true|false|as|where|trait|loop|break|continue|const|static|dyn|move|ref|type|unsafe|async|await|extern|crate|super|in)\b/g, cls: 'kw' },
      { re: /\b(?:println|vec|String|Option|Result|Ok|Err|Box|Vec|std|main)\b/g, cls: 'builtin' },
      { re: /\b[A-Z][A-Za-z0-9_]*\b/g, cls: 'type' },
      { re: /\b\d+\b/g, cls: 'num' },
    ],
  };

  function escapeHtml(text) {
    return text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
  }

  function langFor(el) {
    const match = el.className.match(/language-(\w+)/);
    return match ? match[1] : 'bash';
  }

  function highlight(text, lang) {
    const rules = LANGS[lang] || LANGS.bash;
    const tokens = [{ start: 0, end: text.length, cls: null }];
    let id = 0;

    for (const rule of rules) {
      const next = [];
      for (const token of tokens) {
        if (token.cls) {
          next.push(token);
          continue;
        }

        const slice = text.slice(token.start, token.end);
        let last = 0;
        rule.re.lastIndex = 0;
        let match;

        while ((match = rule.re.exec(slice)) !== null) {
          const start = token.start + match.index;
          const end = start + match[0].length;
          if (start > token.start + last) {
            next.push({ start: token.start + last, end: start, cls: null });
          }
          next.push({ start, end, cls: rule.cls, id: ++id });
          last = match.index + match[0].length;
        }

        if (last === 0) {
          next.push(token);
        } else if (token.start + last < token.end) {
          next.push({ start: token.start + last, end: token.end, cls: null });
        }
      }
      tokens.length = 0;
      tokens.push(...next);
    }

    return tokens
      .map((token) => {
        const chunk = escapeHtml(text.slice(token.start, token.end));
        return token.cls ? `<span class="hl-${token.cls}">${chunk}</span>` : chunk;
      })
      .join('');
  }

  function highlightAll() {
    document.querySelectorAll('pre code[class*="language-"]').forEach((el) => {
      el.innerHTML = highlight(el.textContent, langFor(el));
    });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', highlightAll);
  } else {
    highlightAll();
  }
})();
