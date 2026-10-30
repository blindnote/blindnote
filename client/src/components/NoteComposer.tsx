import { useId, useRef, useState } from 'react'
import { useCopy } from '../lib/i18n'
import { Button } from './Button'
import { Panel } from './Panel'
import { Tagline } from './Tagline'

type NoteComposerProps = {
  value: string
  onChange: (value: string) => void
  /** Form gönderildi (Enter değil, buton ya da Ctrl/Cmd+Enter). Şifreleme/ağ işi çağıranda. */
  onSubmit: () => void
  /** submitting: şifreleme + yükleme sürüyor */
  status: 'idle' | 'submitting'
  /** Gösterilecek hata metni (boş mesaj, ağ hatası vb.). Metni çağıran belirler. */
  error?: string
  /** Gönder butonunu kapatmak için. Boş metinde buton açık kalır; basılınca uyarı çıkar. */
  canSubmit?: boolean
  /** Boş gönderimde gösterilecek uyarı */
  emptyMessage?: string
  maxLength?: number
}

/**
 * Figma: note-composer (Anasayfa). Masaüstünde buton kutunun içinde sağ altta,
 * mobilde kutunun altında tam genişlikte.
 */
export function NoteComposer(props: NoteComposerProps) {
  const t = useCopy()
  const id = useId()
  const errorId = `${id}-error`
  const submitting = props.status === 'submitting'
  const canSubmit = props.canSubmit ?? true
  const isEmpty = props.value.trim().length === 0
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  // Boş gönderim uyarısı (Figma'da yok — türetildi). Yazmaya başlayınca kaybolur.
  const [emptyTried, setEmptyTried] = useState(false)
  // Her başarısız denemede balon yeniden "patlar" (key ile yeniden çizilir)
  const [attempt, setAttempt] = useState(0)
  const showEmpty = emptyTried && isEmpty
  const error = props.error ?? (showEmpty ? (props.emptyMessage ?? t.composer.empty) : undefined)

  const actions = (block: boolean) => (
    <>
      <Button type="submit" blockOnMobile={block} disabled={!canSubmit} loading={submitting} loadingLabel={t.composer.loading}>
        {t.composer.button}
      </Button>
      <Tagline align="right" />
    </>
  )

  return (
    <form
      noValidate
      aria-busy={submitting || undefined}
      onSubmit={(e) => {
        e.preventDefault()
        if (!canSubmit || submitting) return
        setAttempt((n) => n + 1)
        if (isEmpty) {
          setEmptyTried(true)
          textareaRef.current?.focus()
          return
        }
        props.onSubmit()
      }}
      className="relative w-full"
    >
      {/*
        Hata: klasik kırmızı satır yerine, kartın arkasından bakan maymunun konuşma balonu
        (Figma'da yok — türetildi). Ekran okuyucular role="alert" ile duyar.
      */}
      {error && (
        <p
          key={attempt}
          id={errorId}
          role="alert"
          className="absolute bottom-[calc(100%+12px)] left-[calc(50%+32px)] z-10 w-max max-w-[min(15rem,calc(50vw-2.75rem))] origin-bottom-left rounded-2xl rounded-bl-md bg-ink px-3 py-2 text-left text-[13px] leading-5 font-semibold text-white shadow-composer motion-safe:animate-bn-pop md:bottom-[calc(100%+22px)] md:left-[calc(50%+54px)] md:max-w-[17rem] md:px-4 md:py-3 md:text-base dark:bg-accent dark:text-ink"
        >
          {error}
          <span aria-hidden="true" className="absolute -bottom-1.5 left-0 size-3 [clip-path:polygon(0_0,100%_0,0_100%)] bg-ink dark:bg-accent" />
        </p>
      )}
      <Panel>
        <div className="flex flex-col gap-2">
          <label htmlFor={id} className="sr-only">
            {t.composer.label}
          </label>
          <textarea
            id={id}
            ref={textareaRef}
            value={props.value}
            onChange={(e) => {
              setEmptyTried(false)
              props.onChange(e.target.value)
            }}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) e.currentTarget.form?.requestSubmit()
            }}
            readOnly={submitting}
            maxLength={props.maxLength}
            placeholder={t.composer.placeholder}
            aria-invalid={error ? true : undefined}
            aria-describedby={error ? errorId : undefined}
            rows={4}
            spellCheck={false}
            autoComplete="off"
            className="max-h-[50vh] min-h-[98px] w-full resize-none bg-transparent text-base leading-6 break-words text-ink-strong outline-none [field-sizing:content] placeholder:text-muted placeholder:italic md:min-h-[87px] md:text-xl md:leading-6 md:placeholder:not-italic dark:text-sand dark:placeholder:text-muted"
          />
          <div className="hidden flex-col items-end gap-6 pt-4 pb-2 md:flex">{actions(false)}</div>
        </div>
      </Panel>
      <div className="mt-5 flex flex-col items-center gap-4 md:hidden">{actions(true)}</div>
    </form>
  )
}
