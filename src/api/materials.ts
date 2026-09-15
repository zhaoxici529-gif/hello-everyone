import type {
  Material,
  MaterialCategory,
  MaterialDescription,
  MaterialImportOutcome,
  MaterialLibraryInfo,
  MaterialPromptBlock,
  MaterialPurpose,
} from '../types'
import { dbList, dbUpdate } from './db'
import { invokeCommand } from './invoke'

export const materialsRepo = {
  list(): Promise<Material[] | null> {
    return dbList<Material>('materials', { orderBy: 'id', desc: true })
  },

  updateMeta(
    id: number,
    data: { tags?: string; description?: string; category_id?: number | null },
  ): Promise<Material | null> {
    return dbUpdate<Material>('materials', id, data)
  },
}

/** 打开系统文件选择框，返回选中的路径 */
export function pickMaterialFiles(): Promise<string[]> {
  return invokeCommand<string[]>('material_pick')
}

/** 把文件复制进素材库并登记到数据库 */
export function importMaterials(
  paths: string[],
  categoryId?: number | null,
): Promise<MaterialImportOutcome> {
  return invokeCommand<MaterialImportOutcome>('material_import', {
    paths,
    categoryId: categoryId ?? null,
  })
}

/** 删除素材：数据库记录 + 素材库里的文件 */
export function deleteMaterial(id: number, deleteFile = true): Promise<boolean> {
  return invokeCommand<boolean>('material_delete', { id, deleteFile })
}

export function materialCategories(): Promise<MaterialCategory[]> {
  return invokeCommand<MaterialCategory[]>('material_categories')
}

export function createMaterialCategory(name: string): Promise<MaterialCategory> {
  return invokeCommand<MaterialCategory>('material_category_create', { name })
}

export function deleteMaterialCategory(id: number): Promise<boolean> {
  return invokeCommand<boolean>('material_category_delete', { id })
}

export function readMaterialText(id: number): Promise<string> {
  return invokeCommand<string>('material_read_text', { id })
}

/** 视频首帧缩略图：前端抓帧后回传 data URL */
export function saveMaterialThumbnail(id: number, dataUrl: string): Promise<Material> {
  return invokeCommand<Material>('material_save_thumbnail', { id, dataUrl })
}

export function materialLibraryInfo(): Promise<MaterialLibraryInfo> {
  return invokeCommand<MaterialLibraryInfo>('material_library_info')
}

export function materialPromptBlock(
  id: number,
  purpose: MaterialPurpose = 'writing',
): Promise<MaterialPromptBlock> {
  return invokeCommand<MaterialPromptBlock>('material_prompt_block', { id, purpose })
}

/** AI 生成素材描述，并自动写回数据库 */
export function describeMaterial(id: number): Promise<MaterialDescription> {
  return invokeCommand<MaterialDescription>('ai_describe_material', { id })
}
