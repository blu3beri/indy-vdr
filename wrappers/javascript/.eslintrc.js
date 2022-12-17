module.exports = {
  ignorePatterns: ['node_modules', '**/node_modules', '**/build'],
  parser: '@typescript-eslint/parser',
  extends: [
    'eslint:recommended',
    'plugin:import/recommended',
    'plugin:import/typescript',
    'plugin:@typescript-eslint/recommended',
    'plugin:@typescript-eslint/recommended-requiring-type-checking',
    'plugin:prettier/recommended',
  ],
  parserOptions: {
    tsconfigRootDir: __dirname,
    project: [
      './tsconfig.eslint.json',
      'shared/tsconfig.json',
      'react-native/tsconfig.json',
      'nodejs/tsconfig.json',
      'react-native-app/tsconfig.json',
    ],
  },
  settings: {
    'import/extensions': ['.js', '.ts', '.jsx', '.tsx'],
    'import/parsers': {
      '@typescript-eslint/parser': ['.ts', '.tsx'],
    },
    'import/resolver': {
      typescript: {
        extensions: ['.js', '.jsx', '.ts', '.tsx'],
        project: [
          'shared/tsconfig.json',
          'react-native/tsconfig.json',
          'nodejs/tsconfig.json',
          'react-native-app/tsconfig.json',
        ],
        alwaysTryTypes: true,
      },
      node: {
        extensions: ['.js', '.jsx', '.ts', '.tsx'],
        project: [
          'shared/tsconfig.json',
          'react-native/tsconfig.json',
          'nodejs/tsconfig.json',
          'react-native-app/tsconfig.json',
        ],
      },
    },
  },
  rules: {
    '@typescript-eslint/explicit-function-return-type': 'off',
    '@typescript-eslint/explicit-module-boundary-types': 'off',
    '@typescript-eslint/no-use-before-define': ['error', { functions: false, classes: false, variables: true }],
    '@typescript-eslint/explicit-member-accessibility': 'error',
    'no-console': 'error',
    '@typescript-eslint/consistent-type-imports': 'error',
    'import/no-cycle': 'error',
    'import/newline-after-import': ['error', { count: 1 }],
    'import/order': [
      'error',
      {
        groups: ['type', ['builtin', 'external'], 'parent', 'sibling', 'index'],
        alphabetize: {
          order: 'asc',
        },
        'newlines-between': 'always',
      },
    ],
    '@typescript-eslint/no-non-null-assertion': 'error',
    'import/no-extraneous-dependencies': [
      'error',
      {
        devDependencies: false,
      },
    ],
  },
  overrides: [
    {
      files: ['.eslintrc.js', 'babel.config.js', 'metro.config.js'],
      env: {
        node: true,
      },
      rules: {
        '@typescript-eslint/no-var-requires': 'off',
        '@typescript-eslint/no-unsafe-call': 'off',
        '@typescript-eslint/no-unsafe-assignment': 'off',
        '@typescript-eslint/no-unsafe-member-access': 'off',
      },
    },
  ],
}
