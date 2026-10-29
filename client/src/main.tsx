import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './styles/app.css'
import { App } from './App'
import { LocaleProvider } from './lib/i18n'

const root = document.getElementById('root')
if (root) {
  createRoot(root).render(
    <StrictMode>
      <LocaleProvider>
        <App />
      </LocaleProvider>
    </StrictMode>,
  )
}
