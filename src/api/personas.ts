import type {
  Persona,
  PersonaCategory,
  PersonaCategoryInfo,
  PersonaExtraction,
  PersonaFields,
  PersonaPromptBlock,
  PersonaPurpose,
} from '../types'
import { dbDelete, dbInsert, dbList, dbUpdate } from './db'
import { invokeCommand } from './invoke'

export interface PersonaPayload {
  name: string
  category: PersonaCategory
  fields: PersonaFields
}

/** 人物档案的增删改查。明细字段序列化成 JSON 存进 fields 列。 */
export const personasRepo = {
  list(): Promise<Persona[] | null> {
    return dbList<Persona>('personas', { orderBy: 'id', desc: true })
  },

  create(payload: PersonaPayload): Promise<Persona | null> {
    return dbInsert<Persona>('personas', {
      name: payload.name,
      category: payload.category,
      fields: JSON.stringify(payload.fields),
    })
  },

  update(id: number, payload: PersonaPayload): Promise<Persona | null> {
    return dbUpdate<Persona>('personas', id, {
      name: payload.name,
      category: payload.category,
      fields: JSON.stringify(payload.fields),
    })
  },

  remove(id: number): Promise<boolean | null> {
    return dbDelete('personas', id)
  },
}

export function personaCategories(): Promise<PersonaCategoryInfo[]> {
  return invokeCommand<PersonaCategoryInfo[]>('persona_categories')
}

/** 给其它模块用：人物档案 → 提示词块 */
export function personaPromptBlock(
  id: number,
  purpose: PersonaPurpose = 'writing',
): Promise<PersonaPromptBlock> {
  return invokeCommand<PersonaPromptBlock>('persona_prompt_block', { id, purpose })
}

/** AI 从已有文案里提取人物特征 */
export function aiExtractPersona(
  sourceText: string,
  name?: string,
): Promise<PersonaExtraction> {
  return invokeCommand<PersonaExtraction>('ai_extract_persona', {
    sourceText,
    name: name ?? null,
  })
}
