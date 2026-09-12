export type PrintLayerId =
  | 'recipient.postalCode'
  | 'recipient.address1'
  | 'recipient.address2'
  | 'recipient.address3'
  | 'recipient.primaryLast'
  | 'recipient.primaryFirst'
  | 'recipient.honorific'
  | 'recipient.coLast.1'
  | 'recipient.coFirst.1'
  | 'recipient.coHonorific.1'
  | 'recipient.coLast.2'
  | 'recipient.coFirst.2'
  | 'recipient.coHonorific.2'
  | 'recipient.coLast.3'
  | 'recipient.coFirst.3'
  | 'recipient.coHonorific.3'
  | 'sender.postalCode'
  | 'sender.address1'
  | 'sender.address2'
  | 'sender.address3'
  | 'sender.primaryLast'
  | 'sender.primaryFirst'
  | 'sender.coLast.1'
  | 'sender.coFirst.1'
  | 'sender.coLast.2'
  | 'sender.coFirst.2'
  | 'sender.coLast.3'
  | 'sender.coFirst.3'
  | 'sender.coLast.4'
  | 'sender.coFirst.4'

export const ALL_PRINT_LAYER_IDS: PrintLayerId[] = [
  'recipient.postalCode',
  'recipient.address1',
  'recipient.address2',
  'recipient.address3',
  'recipient.primaryLast',
  'recipient.primaryFirst',
  'recipient.honorific',
  'recipient.coLast.1',
  'recipient.coFirst.1',
  'recipient.coHonorific.1',
  'recipient.coLast.2',
  'recipient.coFirst.2',
  'recipient.coHonorific.2',
  'recipient.coLast.3',
  'recipient.coFirst.3',
  'recipient.coHonorific.3',
  'sender.postalCode',
  'sender.address1',
  'sender.address2',
  'sender.address3',
  'sender.primaryLast',
  'sender.primaryFirst',
  'sender.coLast.1',
  'sender.coFirst.1',
  'sender.coLast.2',
  'sender.coFirst.2',
  'sender.coLast.3',
  'sender.coFirst.3',
  'sender.coLast.4',
  'sender.coFirst.4',
]

export const PRINT_LAYER_LABELS: Record<PrintLayerId, string> = {
  'recipient.postalCode': '宛名 郵便番号',
  'recipient.address1': '宛名 住所1',
  'recipient.address2': '宛名 住所2',
  'recipient.address3': '宛名 住所3（建物）',
  'recipient.primaryLast': '宛名 姓',
  'recipient.primaryFirst': '宛名 名',
  'recipient.honorific': '宛名 敬称',
  'recipient.coLast.1': '宛名 連名1 姓',
  'recipient.coFirst.1': '宛名 連名1 名',
  'recipient.coHonorific.1': '宛名 連名1 敬称',
  'recipient.coLast.2': '宛名 連名2 姓',
  'recipient.coFirst.2': '宛名 連名2 名',
  'recipient.coHonorific.2': '宛名 連名2 敬称',
  'recipient.coLast.3': '宛名 連名3 姓',
  'recipient.coFirst.3': '宛名 連名3 名',
  'recipient.coHonorific.3': '宛名 連名3 敬称',
  'sender.postalCode': '差出人 郵便番号',
  'sender.address1': '差出人 住所1',
  'sender.address2': '差出人 住所2',
  'sender.address3': '差出人 住所3（建物）',
  'sender.primaryLast': '差出人 姓',
  'sender.primaryFirst': '差出人 名',
  'sender.coLast.1': '差出人 連名1 姓',
  'sender.coFirst.1': '差出人 連名1 名',
  'sender.coLast.2': '差出人 連名2 姓',
  'sender.coFirst.2': '差出人 連名2 名',
  'sender.coLast.3': '差出人 連名3 姓',
  'sender.coFirst.3': '差出人 連名3 名',
  'sender.coLast.4': '差出人 連名4 姓',
  'sender.coFirst.4': '差出人 連名4 名',
}

export function isPostalLayer(layerId: PrintLayerId): boolean {
  return layerId === 'recipient.postalCode' || layerId === 'sender.postalCode'
}

export function isRecipientCoLastLayer(layerId: PrintLayerId): boolean {
  return /^recipient\.coLast\.\d+$/.test(layerId)
}

export function isSenderCoLastLayer(layerId: PrintLayerId): boolean {
  return /^sender\.coLast\.\d+$/.test(layerId)
}
