import type { PrintLayerId } from '../layout/printLayers'
import { ALL_PRINT_LAYER_IDS, PRINT_LAYER_LABELS } from '../layout/printLayers'
import type { LayerVisibility } from '../types'

type Props = {
  visibility: LayerVisibility
  selectedLayerId: PrintLayerId | null
  onSelectLayer: (id: PrintLayerId) => void
  onToggleVisibility: (id: PrintLayerId, visible: boolean) => void
  onSavePrefs: () => void
  savingPrefs: boolean
  prefsDisabled?: boolean
}

export function PrintLayerPanel({
  visibility,
  selectedLayerId,
  onSelectLayer,
  onToggleVisibility,
  onSavePrefs,
  savingPrefs,
  prefsDisabled,
}: Props) {
  return (
    <div className="print-layer-panel">
      <h2 className="print-layer-panel-title">レイヤー</h2>
      <ul className="print-layer-panel-list">
        {ALL_PRINT_LAYER_IDS.map((id) => {
          const checked = visibility[id] !== false
          const selected = selectedLayerId === id
          return (
            <li key={id}>
              <label
                className={`print-layer-panel-item${selected ? ' print-layer-panel-item-selected' : ''}`}
              >
                <input
                  type="checkbox"
                  checked={checked}
                  onChange={(e) => onToggleVisibility(id, e.target.checked)}
                />
                <button
                  type="button"
                  className="print-layer-panel-label-btn"
                  onClick={() => onSelectLayer(id)}
                >
                  {PRINT_LAYER_LABELS[id]}
                </button>
              </label>
            </li>
          )
        })}
      </ul>
      <button
        type="button"
        className="print-layer-panel-save"
        onClick={onSavePrefs}
        disabled={savingPrefs || prefsDisabled}
      >
        {savingPrefs ? '保存中…' : '調整を保存'}
      </button>
    </div>
  )
}
