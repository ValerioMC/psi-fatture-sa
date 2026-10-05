// The recommended JavaScript, TypeScript and Vue rules. Each override below says why it departs from them.
import js from '@eslint/js'
import globals from 'globals'
import tseslint from 'typescript-eslint'
import pluginVue from 'eslint-plugin-vue'

export default tseslint.config(
  { ignores: ['dist/', 'src-tauri/', 'node_modules/'] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...pluginVue.configs['flat/recommended'],
  {
    files: ['**/*.vue'],
    languageOptions: { parserOptions: { parser: tseslint.parser } },
  },
  {
    languageOptions: { globals: { ...globals.browser } },
  },
  {
    files: ['scripts/**/*.mjs'],
    languageOptions: { globals: { ...globals.node } },
  },
  {
    rules: {
      // Templates use a compact one-line style; layout is a formatter's job, not this linter's.
      'vue/max-attributes-per-line': 'off',
      'vue/singleline-html-element-content-newline': 'off',
      'vue/multiline-html-element-content-newline': 'off',
      'vue/html-self-closing': 'off',
      'vue/html-indent': 'off',
      'vue/html-closing-bracket-newline': 'off',
      'vue/first-attribute-linebreak': 'off',
      // Optional TypeScript props are undefined by design; a default would only restate it.
      'vue/require-default-prop': 'off',
      // ProfileSections edits the reactive form its parent owns; replacing a prop stays forbidden.
      'vue/no-mutating-props': ['error', { shallowOnly: true }],
      // `{ key: _key, ...rest }` drops a field on purpose, as tsc's noUnusedLocals already allows.
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_', varsIgnorePattern: '^_', ignoreRestSiblings: true }],
    },
  },
)
