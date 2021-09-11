import {postIntegrationRequest, postFeedback} from './backend';
import {get, set} from './localStorage';

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
    let polarCode = document.getElementsByClassName('language-polar');
    for (let i = 0; i < polarCode.length; i++) {
      let el = polarCode[i];
      monaco.editor
        .colorize(el.innerText, 'polar', { theme: 'polarTheme' })
        .then(colored => {
          el.innerHTML = colored;
          el.parentNode.classList.add('polar-code-in-here');
        });
    }
  });
});

// hide the search box
window.hideSearch = function(_e) {
  const searchModal = document.getElementById('search-modal');
  if (searchModal.style.display == '') {
    searchModal.style.display = 'none';
  }
};

window.addEventListener('load', () => {
  const searchInput = document.getElementById('search-input');
  searchInput.addEventListener('input', e => window.searchInputKeyUp(e));
  makePromptsUnselectable();
  setRequestedIntegrations();
});

// this handles when the button on the left nav is clicked and it toggles the search box
window.searchButtonClick = function(e) {
  e.preventDefault();
  const searchModal = document.getElementById('search-modal');
  const searchInput = document.getElementById('search-input');
  const searchResultsContainer = document.getElementById(
    'search-results-container'
  );

  if (searchModal.style.display == 'none') {
    searchInput.value = '';
    searchModal.style.display = '';
    searchResultsContainer.innerHTML = '';
  }

  setTimeout(() => searchInput.focus(), 0);
};

import('tinykeys').then(tinykeys => {
  tinykeys.default(window, {
    'Control+KeyK': e => {
      e.preventDefault();
      window.searchButtonClick(e);
    },
    Escape: e => {
      window.hideSearch(e);
    }
  });
});

import('algoliasearch').then(algolia => {
  const searchResult = require('./search-result.handlebars');

  // account from algolia
  const algoliaAccount = 'KROZ8F05YT';
  // read only search key
  const algoliaReadOnlySearchKey = '13594a3b7da482e011ce0ab08fdb4c4d';
  // index name - default to prod index
  let algoliaIndex = 'prod_OSODOCS';
  // load index from meta if this is preview
  const searchindexMeta = document.getElementById('search-index');

  if (searchindexMeta) {
    algoliaIndex = searchindexMeta.content;
  }

  const client = algolia.default(algoliaAccount, algoliaReadOnlySearchKey);
  const index = client.initIndex(algoliaIndex);

  const processHits = function(hits) {
    var results = '';
    var count = 0;

    hits.forEach(element => {
      results += searchResult({
        count: count,
        category: element.section + ' -> ' + element.language,
        title: element.title,
        link: element.permalink
      });
      count += 1;
    });

    const searchResultsContainer = document.getElementById(
      'search-results-container'
    );
    searchResultsContainer.innerHTML = results;
  };

  // this searches for a term without a facet
  const searchTerm = function(term) {
    index
      .search(term, {
        analytics: true,
        hitsPerPage: 20,
        attributesToSnippet: '*:20',
        snippetEllipsisText: '...'
      })
      .then(({ hits }) => {
        processHits(hits);
      });
  };

  // this search for a term WITH a facet
  const searchTermWithFacet = function(term, language) {
    index
      .search(term, {
        analytics: true,
        hitsPerPage: 5,
        attributesToSnippet: '*:20',
        snippetEllipsisText: '...',
        maxValuesPerFacet: 5,
        page: 0,
        facets: ['*', 'language'],
        facetFilters: [['language:' + language]]
      })
      .then(({ hits }) => {
        processHits(hits);
      });
  };

  window.searchInputKeyUp = function(event) {
    const searchInput = document.getElementById('search-input');

    event.preventDefault();
    var term = searchInput.value;

    const facetLanguageMeta = document.getElementById('facet-language');
    var facetLanguage = 'any';

    if (facetLanguageMeta) {
      facetLanguage = facetLanguageMeta.content;
    }

    if (term != '') {
      if (facetLanguage == 'any') {
        searchTerm(term);
      } else {
        searchTermWithFacet(searchInput.value, facetLanguage);
      }
    }
  };
});

const REQUESTED_INTEGRATIONS_KEY = 'integrations';

function setRequestedIntegrations() {
  const requestedIntegrations = JSON.parse(get(REQUESTED_INTEGRATIONS_KEY));

  if (!window.location.href.includes('frameworks') || !requestedIntegrations) {
    return;
  }

  requestedIntegrations.forEach(ri => {
    setRequestedIntegration(ri);
  });
}

function setRequestedIntegration(integration) {
  const buttonId = `request-button-${integration}`;
  const el = document.getElementById(buttonId);
  el.innerText = "Requested!"
  el.setAttribute('disabled', '');
}

window.onRequestIntegration = function(integration) {
  postIntegrationRequest(integration)
    .then(() => {
      let requestedIntegrations = JSON.parse(get(REQUESTED_INTEGRATIONS_KEY));
      if (requestedIntegrations) {
        requestedIntegrations.push(integration);
      } else {
        requestedIntegrations = [integration];
      }
      set(REQUESTED_INTEGRATIONS_KEY, JSON.stringify(requestedIntegrations));

      setRequestedIntegration(integration);
    });
}

function makePromptsUnselectable() {
  const languages = ['bash', 'console'];
  languages.forEach(l => {
    const els = document.querySelectorAll(`code.language-${l}[data-lang="${l}"]`);
    els.forEach(el => {
      const newHtml = el.innerHTML.replace(/^\$ /gm, '<span style="user-select:none">$ </span>');
      el.innerHTML = newHtml;
    });
  })
};

window.recordFeedback = (isUp) => {
  postFeedback(isUp).then(() => {
    const upEl = document.getElementById('feedback-up');
    const downEl = document.getElementById('feedback-down');
    if (isUp) {
      upEl.setAttribute('disabled', '');
      downEl.removeAttribute('disabled');
    } else {
      upEl.removeAttribute('disabled');
      downEl.setAttribute('disabled', '');
    }
  });
}

// Support for code tab groups -- hacks ahead!
window.addEventListener("load", () => {
  const unique = (array) =>
    array.filter((value, index, self) => self.indexOf(value) === index);

  const tabGroupDivs = Array.from(
    document.querySelectorAll("div[data-tabgroup]")
  );

  let tabGroups = tabGroupDivs.map(
    (div) => div.attributes["data-tabgroup"].value
  );
  tabGroups = unique(tabGroups);

  for (const tabGroup of tabGroups) {
    console.log(tabGroup);
    const codeBlocks = document.querySelectorAll(
      `div.code[data-tabgroup='${tabGroup}'`
    );
    const first = codeBlocks[0];
    // tabGroupContainer is the full containing div, and replaces the first code
    // block in the tabGroup
    const tabGroupContainer = document.createElement("div");
    tabGroupContainer.className = "code";
    first.parentNode.insertBefore(tabGroupContainer, first);
    // tabContainer contains the clickable tabs
    const tabContainer = document.createElement("div");
    // codeContainer contains the, well, code
    const codeContainer = document.createElement("div");
    tabGroupContainer.appendChild(tabContainer);
    tabGroupContainer.appendChild(codeContainer);
    tabContainer.className =
      "filename rounded-t-md bg-gray-200 text-gray-700 text-sm flex";

    const pres = Array.from(codeBlocks).map((div) => div.querySelector("pre"));
    const filenames = Array.from(codeBlocks).map((div) => {
      const newDiv = document.createElement("div");
      newDiv.replaceChildren(...div.querySelector(".filename").childNodes);
      return newDiv;
    });

    codeBlocks.forEach((block) => block.remove());
    tabContainer.replaceChildren(...filenames);

    const unselectedTabStyle = {
      padding: "0.6rem 0.4rem",
      display: "flex",
      alignItems: "center",
      cursor: "pointer",
      fontWeight: "normal",
      boxShadow: "none",
    };

    const selectedTabStyle = {
      ...unselectedTabStyle,
      fontWeight: "bold",
      boxShadow: "inset 0 -2px 0 #666",
    };

    function unselectFilename(filename) {
      Object.assign(filename.style, unselectedTabStyle);
    }

    function selectFilename(filename) {
      Object.assign(filename.style, selectedTabStyle);
    }

    function select(selectedIndex) {
      filenames.forEach(unselectFilename);
      selectFilename(filenames[selectedIndex]);
      codeContainer.replaceChildren(pres[selectedIndex]);
    }

    filenames.forEach((filename, i) => {
      filename.addEventListener("click", () => select(i));
    });


    select(0);
  }
});