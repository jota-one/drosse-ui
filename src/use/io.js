import { invoke } from '@tauri-apps/api/tauri'
import { commands } from '@/config'

export default function useIo() {
  const browse = async dir => {
    const entries = await invoke(commands.browse, { dir })
    return entries
      .sort((a, b) => a.path.toLowerCase() > b.path.toLowerCase() ? 1 : -1)
  }

  const fetchConfig = async drosse => {
    const hosts = drosse.hosts
    const port = drosse.port
    const proto = drosse.proto

    try {
      const response = await fetch(`${proto}://${hosts[0]}:${port}/UI`)
      return response.json()
    } catch (e) {
      // eslint-disable-next-line
      console.error(e)
    }
  }

  const fetchDrosses = () => invoke(commands.list)

  const fetchHandler = async (drosse, file) => {
    try {  
      const response = await invoke(commands.file, {
        uuid: { ...drosse.uuid },
        file
      })

      return {
        content: response.content,
        language: file.endsWith('.json') ? 'json' : 'javascript',
      }
    } catch (e) {
      return {
        language: 'text',
        content: `Failed loading file ${file}`,
      }
    }
  }

  const importFolder = path => invoke(commands.import, { path })
  const openFile = (uuid, file) => invoke(commands.open, { uuid, file })
  const saveDrosses = drosses => invoke(commands.save, { ...drosses })
  const start = uuid => invoke(commands.start, { uuid })
  const stop = uuid => invoke(commands.stop, { uuid })

  return {
    browse,
    fetchConfig,
    fetchDrosses,
    fetchHandler,
    importFolder,
    openFile,
    saveDrosses,
    start,
    stop,
  }
}
