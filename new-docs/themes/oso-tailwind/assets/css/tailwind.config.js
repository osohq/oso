const colors = require('tailwindcss/colors');

osoBlue = {
  100: '#e1dbf6',
  200: '#c0b5f3',
  300: '#a594ee',
  400: '#745dd6',
  500: '#432caa',
  600: '#321e8b',
  700: '#211164',
  800: '#170950',
  900: '#0e024e',
};
osoYellow = {
  50: '#fefce8',
  100: '#fef9c3',
  200: '#fdf29f',
  300: '#ffea73',
  400: '#fadb2b',
  500: '#ffd803',
  600: '#bd8f13',
  700: '#a16207',
  800: '#854d0e',
  900: '#713f12',
};

function withShadeNames(palette) {
  return {
    ...palette,
    lightest: palette[100],
    light: palette[300],
    DEFAULT: palette[500],
    dark: palette[700],
    darkest: palette[900],
  };
}

module.exports = {
  theme: {
    colors: {
      black: colors.black,
      white: colors.white,
      gray: colors.coolGray,
      red: withShadeNames(colors.red),
      green: withShadeNames(colors.green),
      blue: withShadeNames(colors.blue),
      orange: withShadeNames(colors.orange),
      primary: withShadeNames(osoBlue),
      yellow: withShadeNames(osoYellow),
    },
    extend: {},
  },
  variants: {},
  plugins: [require('@tailwindcss/typography')],
};
