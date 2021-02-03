const monaco = import('monaco-editor');

// Basic navigation functionality
document.querySelector('button > svg').addEventListener('click', function(e) {
  e.stopPropagation();
  e.preventDefault();
});

const navContent = document.getElementById('nav-content');
const navButton = document.getElementById('nav-toggle');
const navToggleOpen = document.getElementById('nav-toggle-open');
const navToggleClosed = document.getElementById('nav-toggle-closed');
const navTitle = document.getElementById('header-page-title');
navButton.addEventListener('click', () => {
  navContent.classList.toggle('hidden');
  navTitle.classList.toggle('hidden');
  navToggleOpen.classList.toggle('hidden');
  navToggleClosed.classList.toggle('hidden');
});

const sideBarContent = document.getElementById('sidebar-content');
const sideBarButton = document.getElementById('sidebar-toggle');
const sideBarSearch = document.getElementById('sidebar-search');
if (sideBarButton) {
  const toggleSideBar = () => sideBarContent.classList.toggle('hidden');
  sideBarButton.addEventListener('click', toggleSideBar);
}

const langButton = document.getElementById('language-selector-toggle');
const langsideBar = document.getElementById('language-selector-content');
langButton.addEventListener('click', () =>
  langsideBar.classList.toggle('hidden')
);

// Close dropdown sideBars if the user clicks outside of them
window.onclick = function(event) {
  console.log(event.target);
  switch (event.target) {
    case navButton:
      break;
    case sideBarButton:
      break;
    case sideBarSearch:
      break;
    case langButton:
      break;

    default:
      // default to hidden
      var contents = [navContent, langsideBar, sideBarContent, navToggleOpen];

      for (content of contents) {
        if (content && !content.classList.contains('hidden')) {
          content.classList.toggle('hidden');
        }
      }

      // default to visible
      var contents = [navTitle, navToggleClosed];

      for (content of contents) {
        if (content && content.classList.contains('hidden')) {
          content.classList.toggle('hidden');
        }
      }
      break;
  }
};

//Get the button:
mybutton = document.getElementById('scroll-to-top');

// When the user scrolls down 20px from the top of the document, show the button
window.onscroll = function() {
  scrollFunction();
};

function scrollFunction() {
  if (document.body.scrollTop > 20 || document.documentElement.scrollTop > 20) {
    mybutton.style.display = 'block';
  } else {
    mybutton.style.display = 'none';
  }
}

// When the user clicks on the button, scroll to the top of the document
function topFunction() {
  document.body.scrollTop = 0; // For Safari
  document.documentElement.scrollTop = 0; // For Chrome, Firefox, IE and Opera
}

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
  operators: ['=', '>', '>=', '<', '<=', '==', '!=', '+', '-', '*', '/', '?='],
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
