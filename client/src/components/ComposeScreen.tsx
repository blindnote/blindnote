import { Hero } from './Hero'
import { NoteComposer } from './NoteComposer'

type ComposeScreenProps = {
  value: string
  onChange: (value: string) => void
  onSubmit: () => void
  status: 'idle' | 'submitting'
  error?: string
  maxLength?: number
}

/** Ekran (a) Yaz — Figma: Anasayfa */
export function ComposeScreen(props: ComposeScreenProps) {
  return (
    <Hero
      title="Özgürce Paylaş"
      subtitle="Açıldığında kendini yok edecek bağlantı oluşturun."
      monkey={{ pose: 'eyes', placement: 'peek' }}
      card={
        <NoteComposer
          value={props.value}
          onChange={props.onChange}
          onSubmit={props.onSubmit}
          status={props.status}
          error={props.error}
          maxLength={props.maxLength}
        />
      }
    />
  )
}
