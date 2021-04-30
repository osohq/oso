// import('oso')
//   .catch(e => console.error('Error importing `oso`:', e))
//   .then(m => (window.oso = m));

import('monaco-editor-core').then(monaco => {
  // Monokai colors
  const COLOR = {
    GHOST_WHITE: 'F8F8F0',
    LIGHT_GHOST_WHITE: 'F8F8F2',
    LIGHT_GRAY: 'CCCCCC',
    GRAY: '888888',
    BROWN_GRAY: '49483E',
    DARK_GRAY: '282828',

    YELLOW: 'E6DB74',
    BLUE: '66D9EF',
    PINK: 'F92672',
    PURPLE: 'AE81FF',
    BROWN: '75715E',
    ORANGE: 'FD971F',
    LIGHT_ORANGE: 'FFD569',
    GREEN: 'A6E22E',
    SEA_GREEN: '529B2F'
  };

  monaco.languages.register({
    id: 'polar'
  });

  monaco.languages.setMonarchTokensProvider('polar', {
    keywords: [
      'and',
      'cut',
      'debug',
      'forall',
      'if',
      'in',
      'matches',
      'new',
      'not',
      'or',
      'print'
    ],
    operators: [
      '=',
      '>',
      '>=',
      '<',
      '<=',
      '==',
      '!=',
      '+',
      '-',
      '*',
      '/',
      '?='
    ],
    ignoreCase: true,
    defaultToken: 'invalid',
    brackets: [
      ['{', '}', 'delimiter.curly'],
      ['[', ']', 'delimiter.square'],
      ['(', ')', 'delimiter.parenthesis']
    ],
    keywordops: ['::'],
    symbols: /[=><!~?:&|+\-*\/\^%]+/,
    escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,
    identifier: /[a-z_][\w]*/,
    tokenizer: {
      root: [
        [
          /(@identifier)(?=\([^)]*\))/,
          {
            cases: {
              '$1@keywords': {
                cases: {
                  debug: 'keyword.debug',
                  print: 'keyword.print',
                  '@default': 'keyword'
                }
              },
              '@default': 'predicate'
            }
          }
        ],

        [
          /@identifier/,
          {
            cases: {
              '@keywords': {
                cases: {
                  debug: 'keyword.debug',
                  '@default': 'keyword'
                }
              },
              '@default': { token: 'identifier' }
            }
          }
        ],

        // whitespace
        { include: '@whitespace' },

        // delimiters and operators
        [/[{}()\[\]]/, '@brackets'],
        [
          /@symbols/,
          {
            cases: {
              '@keywordops': 'keyword',
              '@operators': 'operator',
              '@default': ''
            }
          }
        ],

        // numbers
        [/\d*\.\d+([eE][\-+]?\d+)?/, 'number.float'],
        [/\d+/, 'number'],

        // delimiter: after number because of .\d floats
        [/\./, 'delimiter.dot'],
        [/;/, 'delimiter.semicolon'],

        // strings
        [/"([^"\\]|\\.)*$/, 'string.invalid'], // non-terminated string
        [/"/, { token: 'string.quote', bracket: '@open', next: '@string' }]
      ],

      string: [
        [/[^\\"]+/, 'string'],
        [/@escapes/, 'string.escape'],
        [/\\./, 'string.escape.invalid'],
        [/"/, { token: 'string.quote', bracket: '@close', next: '@pop' }]
      ],

      whitespace: [[/[ \t\r\n]+/, 'white'], [/#.*$/, 'comment']]
    }
  });

  // Define a new theme that constains only rules that match this language
  monaco.editor.defineTheme('polarTheme', {
    base: 'vs-dark',
    rules: [
      { token: 'keyword.debug', foreground: COLOR.GREEN, fontStyle: 'italic' },
      { token: 'keyword.print', foreground: COLOR.GREEN, fontStyle: 'italic' },
      { token: 'keyword', foreground: COLOR.PINK },
      { token: 'operator', foreground: COLOR.PINK },
      { token: 'number', foreground: COLOR.PURPLE },
      { token: 'string', foreground: COLOR.YELLOW },
      { token: 'comment', foreground: COLOR.BROWN },
      { token: 'delimiter', foreground: COLOR.SEA_GREEN },
      { token: 'predicate', foreground: COLOR.BLUE, fontStyle: 'italic' },
      { token: '', foreground: COLOR.LIGHT_GHOST_WHITE }
    ]
  });

  monaco.editor.setTheme('polarTheme');

  window.addEventListener('load', () => {
    const literalIncludes = document.querySelectorAll(
      'div.language-polar > div.highlight code.language-plaintext'
    );
    const backtickBlocks = document.querySelectorAll('code.language-polar');
    const polarSnippets = [...literalIncludes, ...backtickBlocks];
    for (const el of polarSnippets) {
      let spans = el.children;
      let spanStyles = {};
      for (let i = 0; i < spans.length; i++) {
        spanStyles[normalizeSpaces(spans[i].innerText.trimEnd())] =
          spans[i].style.cssText;
      }
      monaco.editor
        .colorize(el.innerText, 'polar', { theme: 'polarTheme' })
        .then(colored => {
          el.innerHTML = colored;
          el.parentNode.classList.add('polar-code-in-here');
          let highlightChildren = el.children;
          for (let i = 0; i < highlightChildren.length; i++) {
            let text = highlightChildren[i].innerText.trimEnd();
            if (text.indexOf('implies') != -1) {
              console.log(text);
              console.log(Object.getOwnPropertyNames(spanStyles)[0]);
              console.log(Object.getOwnPropertyNames(spanStyles)[0] == text);
            }
            let style = spanStyles[highlightChildren[i].innerText.trimEnd()];
            if (typeof style !== 'undefined') {
              console.log('set style');
              highlightChildren[i].setAttribute('style', style);
            }

            if (
              highlightChildren[i].tagName === 'SPAN' &&
              highlightChildren[i].children[0].tagName == 'SPAN' &&
              highlightChildren[i].innerText == ''
            ) {
              // THere are random empty spans that show up in chrome. Trying to remove them proves difficult.
              // It's fine in safari.
            }
          }
        });
    }
  });
});

// It seems that monaco makes spaces U+00A0 (160) instead of U+0020 (32).
function normalizeSpaces(str) {
  return str.replaceAll(String.fromCharCode(32), String.fromCharCode(160));
}
