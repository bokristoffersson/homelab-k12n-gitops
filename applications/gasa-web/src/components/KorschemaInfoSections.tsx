import type { Assessment, BeforeYouStart } from '../types'

interface Props {
  beforeYouStart?: BeforeYouStart
  assessment?: Assessment
}

export default function KorschemaInfoSections({
  beforeYouStart,
  assessment,
}: Props) {
  return (
    <>
      {beforeYouStart && (
        <details className="info-section">
          <summary>{beforeYouStart.title}</summary>
          <div className="info-body">
            <ul className="info-list">
              {beforeYouStart.requirements.map((req) => (
                <li key={req.title}>
                  <strong>{req.title}.</strong> {req.text}
                </li>
              ))}
            </ul>
            <h4>Före provet</h4>
            <ul className="info-list">
              {beforeYouStart.beforeExam.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </div>
        </details>
      )}
      {assessment && (
        <details className="info-section">
          <summary>{assessment.title}</summary>
          <div className="info-body">
            <p>{assessment.intro}</p>
            <ul className="info-list">
              {assessment.areas.map((area) => (
                <li key={area.title}>
                  <strong>{area.title}.</strong> {area.text}
                </li>
              ))}
            </ul>
            <p>{assessment.outro}</p>
          </div>
        </details>
      )}
    </>
  )
}
