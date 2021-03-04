// import('oso')
//   .catch(e => console.error('Error importing `oso`:', e))
//   .then(m => (window.oso = m));

import('monaco-editor-core').then((monaco) => {
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
    SEA_GREEN: '529B2F',
  };

  monaco.languages.register({
    id: 'polar',
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
      'print',
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
      '?=',
    ],
    ignoreCase: true,
    defaultToken: 'invalid',
    brackets: [
      ['{', '}', 'delimiter.curly'],
      ['[', ']', 'delimiter.square'],
      ['(', ')', 'delimiter.parenthesis'],
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
                  '@default': 'keyword',
                },
              },
              '@default': 'predicate',
            },
          },
        ],

        [
          /@identifier/,
          {
            cases: {
              '@keywords': {
                cases: {
                  debug: 'keyword.debug',
                  '@default': 'keyword',
                },
              },
              '@default': { token: 'identifier' },
            },
          },
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
              '@default': '',
            },
          },
        ],

        // numbers
        [/\d*\.\d+([eE][\-+]?\d+)?/, 'number.float'],
        [/\d+/, 'number'],

        // delimiter: after number because of .\d floats
        [/\./, 'delimiter.dot'],
        [/;/, 'delimiter.semicolon'],

        // strings
        [/"([^"\\]|\\.)*$/, 'string.invalid'], // non-terminated string
        [/"/, { token: 'string.quote', bracket: '@open', next: '@string' }],
      ],

      string: [
        [/[^\\"]+/, 'string'],
        [/@escapes/, 'string.escape'],
        [/\\./, 'string.escape.invalid'],
        [/"/, { token: 'string.quote', bracket: '@close', next: '@pop' }],
      ],

      whitespace: [
        [/[ \t\r\n]+/, 'white'],
        [/#.*$/, 'comment'],
      ],
    },
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
      { token: '', foreground: COLOR.LIGHT_GHOST_WHITE },
    ],
  });

  monaco.editor.setTheme('polarTheme');

  window.addEventListener('load', () => {
    let polarCode = document.getElementsByClassName('language-polar');
    for (let i = 0; i < polarCode.length; i++) {
      let el = polarCode[i];
      monaco.editor
        .colorize(el.innerText, 'polar', { theme: 'polarTheme' })
        .then((colored) => {
          el.innerHTML = colored;
          el.parentNode.classList.add('polar-code-in-here');
        });
    }
  });
});

import('lunr').then(({ default: lunr }) => {
  // Search
  window.addEventListener(
    'load',
    (_event) => {
      let index = null;
      let lookup = {};
      let queuedTerm = null;

      let previousContent = null;
      let previousX = 0;
      let previousY = 0;
      let previousSearch = null;

      var form = document.getElementById('sidebar-search-form');
      var input = document.getElementById('sidebar-search-input');

      form.addEventListener(
        'submit',
        function (event) {
          event.preventDefault();

          var term = input.value.trim();
          if (!term) return;

          startSearch(term);
        },
        false
      );

      function startSearch(term) {
        // Start icon animation.
        form.setAttribute('data-running', 'true');

        if (index) {
          // Index already present, search directly.
          search(term);
        } else if (queuedTerm) {
          // Index is being loaded, replace the term we want to search for.
          queuedTerm = term;
        } else {
          // Start loading index, perform the search when done.
          queuedTerm = term;
          initIndex();
        }
      }

      function searchDone() {
        // Stop icon animation.
        form.removeAttribute('data-running');

        queuedTerm = null;
      }

      async function initIndex() {
        try {
          const searchIndex = await fetch('/search.json');
          const jsonIndex = await searchIndex.json();
          index = lunr(function () {
            this.ref('uri');

            // If you added more searchable fields to the search index, list them here.
            this.field('title');
            this.field('content');

            this.metadataWhitelist = ['index', 'position'];

            for (const doc of jsonIndex) {
              this.add(doc);
              lookup[doc.uri] = doc;
            }
          });

          // Search index is ready, perform the search now
          search(queuedTerm);
        } catch (e) {
          // TODO(gj): maybe disable the search bar if this fails?
          console.error(e);
        }
      }

      function search(term) {
        const results = index.search(term);

        if (previousContent === null) {
          // The element where search results should be displayed, adjust as needed.
          previousContent = document.getElementById('content-wrapper');
          previousX = window.pageXOffset;
          previousY = window.pageYOffset;
        }

        const outermost = document.createElement('div');
        outermost.classList.add(
          'min-w-0',
          'w-full',
          'flex-auto',
          'lg:static',
          'lg:max-h-full',
          'lg:overflow-visible'
        );

        const outer = document.createElement('div');
        outer.classList.add('w-full', 'flex');

        const searchContainer = document.createElement('div');
        searchContainer.classList.add(
          'prose',
          'min-w-0',
          'flex-auto',
          'px-4',
          'sm:px-6',
          'xl:px-8',
          'pt-6',
          'pb-24',
          'lg:pb-16'
        );

        outermost.appendChild(outer);
        outer.appendChild(searchContainer);

        // Hide old content.
        previousContent.setAttribute('hidden', true);

        window.scrollTo(0, 0);

        if (previousSearch !== null) {
          previousSearch.remove();
        }

        previousSearch = outermost;

        // Insert new content.
        previousContent.insertAdjacentElement('afterend', outermost);

        const returnToContent = document.createElement('a');
        const returnToContentText = document.createElement('h2');
        returnToContentText.textContent = 'Return to content.';
        returnToContent.appendChild(returnToContentText);
        returnToContent.onclick = () => {
          outermost.remove();
          previousContent.removeAttribute('hidden');
          window.scrollTo(previousX, previousY);
          previousContent = null;
          previousX = 0;
          previousY = 0;
          previousSearch = null;
        };
        searchContainer.appendChild(returnToContent);

        const title = document.createElement('h1');
        title.id = 'search-results-heading';
        title.className = 'list-title';
        if (results.length === 0) {
          title.textContent = `No results found for "${term}".`;
        } else if (results.length === 1) {
          title.textContent = `Found one result for "${term}".`;
        } else {
          title.textContent = `Found ${results.length} results for "${term}".`;
        }
        searchContainer.appendChild(title);

        const template = document.getElementById('search-result');
        for (const result of results) {
          const doc = lookup[result.ref];

          // Fill out search result template, adjust as needed.
          const element = template.content.cloneNode(true);
          element.querySelector(
            '.summary-title-link'
          ).href = element.querySelector('.read-more-link').href = doc.uri;
          element.querySelector('.summary-title-link').textContent = doc.title;

          const positions = Object.values(result.matchData.metadata)
            .filter((m) => m.content)
            .flatMap((m) => m.content.position);

          const firstOccurrence = positions.reduce(
            (earliest, [challenger, _]) =>
              challenger <= earliest ? challenger : earliest,
            doc.content.length - 1
          );

          let truncated;
          let leftTrimmed = 1;
          if (doc.content.length <= 200) {
            truncated = doc.content;
          } else if (firstOccurrence <= 100) {
            truncated = doc.content.slice(0, 200) + '…';
          } else if (firstOccurrence + 100 >= doc.content.length) {
            leftTrimmed = doc.content.length - 200;
            truncated = '…' + doc.content.slice(leftTrimmed);
          } else {
            leftTrimmed = firstOccurrence - 100;
            truncated =
              '…' + doc.content.slice(leftTrimmed, firstOccurrence + 100) + '…';
          }

          const highlighted = highlightOccurrences(
            truncated,
            positions,
            leftTrimmed
          );

          element.querySelector('.summary').innerHTML = highlighted;
          searchContainer.appendChild(element);
        }

        searchDone();
      }

      function highlightOccurrences(text, positions, leftTrimmed) {
        return positions.reduce((text, [from, len], index) => {
          const start = from + index * '<mark></mark>'.length - leftTrimmed + 1;
          const before = text.slice(0, start);
          const target = text.slice(start, start + len);
          const after = text.slice(start + len);
          return `${before}<mark>${target}</mark>${after}`;
        }, text);
      }
    },
    false
  );
});
