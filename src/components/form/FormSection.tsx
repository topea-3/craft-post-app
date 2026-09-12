type Props = {
  title: string
  description?: string
  children: React.ReactNode
}

export function FormSection({ title, description, children }: Props) {
  return (
    <section className="form-section">
      <header className="form-section-header">
        <h2 className="form-section-title">{title}</h2>
        {description ? <p className="form-section-description">{description}</p> : null}
      </header>
      <div className="form-section-body">{children}</div>
    </section>
  )
}

