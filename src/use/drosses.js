import { listen } from '@tauri-apps/api/event'
import { computed, ref } from 'vue'
import bus from '@/bus'
import { events } from '@/config'
import useRoutes from './routes'
import useIo from './io'

const { getRoutes } = useRoutes()
const { fetchConfig, fetchDrosses, saveDrosses } = useIo()

const drosses = ref({})
const loaded = false

listen(events.up, async e => {
  const data = e.payload
  const uuid = data.drosse.uuid
  drosses.value[uuid] = data.drosse

  const config = await fetchConfig(data.drosse)

  if (config) {
    drosses.value[uuid].routes = getRoutes(config, drosses.value[uuid].routes)
    saveDrosses(drosses.value)
  } else {
    drosses.value[uuid].up = false
    saveDrosses(drosses.value)
  }
})

listen(events.down, e => {
  drosses.value[e.payload.uuid].up = false
})

listen(events.request, e => {
  const { method, url, uuid } = e.payload.request
  bus.emit('request', { uuid, method, url })
})

listen(events.log, e => {
  const { uuid, msg } = e.payload
  bus.emit('log', { uuid, msg })
})

export default function useDrosses() {
  const loadDrosses = async () => {
    if (loaded) {
      return drosses
    }

    const _drosses = await fetchDrosses()

    if (_drosses) {
      drosses.value = _drosses
    }

    return drosses
  }

  const openDrosse = uuid => {
    for (const uuid of Object.keys(drosses.value)) {
      drosses.value[uuid].selected = false
    }

    drosses.value[uuid].open = true
    drosses.value[uuid].selected = true
    saveDrosses(drosses.value)
  }

  const closeDrosse = uuid => {
    drosses.value[uuid].open = false
    drosses.value[uuid].selected = false
    saveDrosses(drosses.value)
  }

  const openHome = () => {
    for (const uuid of Object.keys(drosses.value)) {
      drosses.value[uuid].selected = false
    }
    saveDrosses(drosses.value)
  }

  const viewHome = computed(
    () => !Object.values(drosses.value).some(drosse => drosse.selected)
  )

  return {
    drosses,
    loadDrosses,
    openDrosse,
    closeDrosse,
    openHome,
    viewHome,
  }
}
