<script setup lang="ts">
import {onUnmounted, ref, watch} from 'vue'
import {IInputEmits, IInputProps} from '@/components/ui/tgui/inputs/tgui-input.types.ts'
import {
  clamp,
  sanitizeHexInput,
  useColorPicker
} from '@/components/ui/tgui/inputs/color-picker/color.helpers.ts'
import {Tabs, TabsContent, TabsList, TabsTrigger} from '@/components/ui/tabs'
import {Input} from '@/components/ui/input'
import {InputGroup, InputGroupAddon, InputGroupInput} from "@/components/ui/input-group";

const props = defineProps<IInputProps>()
const emit = defineEmits<IInputEmits>()

const {
  hue, saturation, brightness, alpha, format,
  currentHexWithAlpha, hueColor, currentRgba,
  hexInput, applyHex, rgbInputs, applyRgb, hslInputs, applyHsl,
  applyFromHex, applyPreset,
} = useColorPicker()

applyFromHex(props.modelValue)
watch(currentHexWithAlpha, (val) => emit('update:modelValue', val))


const areaEl = ref<HTMLElement | null>(null)
const hueEl = ref<HTMLElement | null>(null)
const alphaEl = ref<HTMLElement | null>(null)

type DragTarget = 'area' | 'hue' | 'alpha' | null
let dragging: DragTarget = null

function handleMove(e: MouseEvent) {
  const x = e.clientX
  const y = e.clientY
  if (dragging === 'area' && areaEl.value) {
    const r = areaEl.value.getBoundingClientRect()
    saturation.value = clamp((x - r.left) / r.width)
    brightness.value = clamp(1 - (y - r.top) / r.height)
  }
  if (dragging === 'hue' && hueEl.value) {
    const r = hueEl.value.getBoundingClientRect()
    hue.value = clamp((x - r.left) / r.width) * 360
  }
  if (dragging === 'alpha' && alphaEl.value) {
    const r = alphaEl.value.getBoundingClientRect()
    alpha.value = clamp((x - r.left) / r.width)
  }
}

function stopDrag() {
  dragging = null
  window.removeEventListener('mousemove', handleMove)
  window.removeEventListener('mouseup', stopDrag)
}

function startDrag(target: DragTarget, e: MouseEvent) {
  dragging = target
  handleMove(e)
  window.addEventListener('mousemove', handleMove)
  window.addEventListener('mouseup', stopDrag)
}

onUnmounted(() => {
  if (dragging) stopDrag()

})

const presets = [
  '#ef4444', '#f97316', '#f59e0b', '#84cc16',
  '#10b981', '#06b6d4', '#3b82f6', '#8b5cf6',
  '#ec4899', '#ffffff', '#94a3b8', '#475569',
  '#1e293b', '#020617', '#713f12', '#14532d',
]
</script>

<template>
  <div class="select-none border border-input p-4 space-y-3.5">
    <div>
      <p class="text-[10px] font-medium uppercase tracking-widest text-muted-foreground mb-2">
        Presets</p>
      <div class="grid grid-cols-8 gap-1.5">
        <button
            v-for="color in presets"
            :key="color"
            class="aspect-square w-full rounded-xl border border-border/60 hover:scale-110 transition-all duration-150"
            :style="{ background: color }"
            :title="color"
            @click="applyPreset(color)"
        />
      </div>
    </div>
    <div
        ref="areaEl"
        class="relative h-44 w-full rounded-xl cursor-crosshair overflow-hidden"
        :style="{
        background: `
          linear-gradient(to top, #000 0%, transparent 100%),
          linear-gradient(to right, #fff 0%, ${hueColor} 100%)
        `
      }"
        @mousedown.prevent="(e) => startDrag('area', e)"
    >
      <div
          class="absolute w-4.5 h-4.5 -translate-x-1/2 -translate-y-1/2 rounded-full border-[2.5px] border-white ring-1 ring-black/20 shadow-lg pointer-events-none"
          :style="{ left: `${saturation * 100}%`, top: `${(1 - brightness) * 100}%`, background: currentHexWithAlpha }"
      />
    </div>

    <div class="flex gap-3 items-center">
      <div
          class="relative shrink-0 w-9 h-9 rounded-lg border border-border shadow-inner overflow-hidden">
        <div class="absolute inset-0 rounded-lg" :style="{ background: currentRgba }"/>
      </div>

      <div class="flex-1 space-y-2">
        <div
            ref="hueEl"
            class="relative h-3 w-full rounded-full cursor-pointer"
            style="background: linear-gradient(to right, #ff0000,#ffff00,#00ff00,#00ffff,#0000ff,#ff00ff,#ff0000)"
            @mousedown.prevent="(e) => startDrag('hue', e)"
        >
          <div
              class="absolute top-1/2 -translate-x-1/2 -translate-y-1/2 w-4 h-4 rounded-full border-2 border-white shadow-md ring-1 ring-black/20 pointer-events-none"
              :style="{ left: `${(hue / 360) * 100}%`, background: hueColor }"
          />
        </div>

        <div
            ref="alphaEl"
            class="relative h-3 w-full rounded-full cursor-pointer overflow-hidden"
            @mousedown.prevent="(e) => startDrag('alpha', e)"
        >
          <div
              class="absolute inset-0 rounded-full"
              :style="{ background: `linear-gradient(to right, transparent, ${currentHexWithAlpha.slice(0, 7)})` }"
          />

          <div
              class="absolute top-1/2 -translate-x-1/2 -translate-y-1/2 w-4 h-4 rounded-full border-2 border-white shadow-md ring-1 ring-black/20 pointer-events-none"
              :style="{ left: `${alpha * 100}%`, background: currentRgba }"
          />
        </div>
      </div>
    </div>

    <Tabs v-model="format">
      <div class="flex items-center gap-2">
        <TabsList class="flex-1 grid grid-cols-3 bg-muted/70">
          <TabsTrigger value="hex">Hex</TabsTrigger>
          <TabsTrigger value="rgb">RGB</TabsTrigger>
          <TabsTrigger value="hsl">HSL</TabsTrigger>
        </TabsList>
      </div>
      <TabsContent value="hex">
        <Input
            v-model="hexInput"
            maxlength="9"
            placeholder="#000000"
            class="tracking-wider"
            @input="(e: Event) => { hexInput = sanitizeHexInput((e.target as HTMLInputElement).value) }"
            @blur="applyHex"
            @keyup.enter="applyHex"
        />
        <p class="mt-1 text-[10px] text-muted-foreground">
          #rgb, #rrggbb, #rgba or #rrggbbaa
        </p>
      </TabsContent>
      <TabsContent value="rgb">
        <div class="grid grid-cols-4 gap-1.5">
          <div v-for="ch in (['r', 'g', 'b', 'a'] as const)" :key="ch" class="flex flex-col gap-1">
            <InputGroup>
              <InputGroupInput
                  v-model.number="rgbInputs[ch]"
                  type="number"
                  min="0"
                  :max="ch === 'a' ? 100 : 255"
                  :key="ch"
                  class="px-1 text-center text-md"
                  @blur="applyRgb"
                  @keyup.enter="applyRgb"
              />
              <InputGroupAddon class="p-1 border-r">
                {{ch.toUpperCase()}}
              </InputGroupAddon>
            </InputGroup>
          </div>
        </div>
      </TabsContent>
      <TabsContent value="hsl">
        <div class="grid grid-cols-4 gap-1.5">
          <div v-for="(max, ch) in ({ h: 360, s: 100, l: 100, a: 100 } as const)" :key="ch"
               class="flex flex-col gap-1">
            <InputGroup>
              <InputGroupInput
                  v-model.number="hslInputs[ch]"
                  type="number" min="0" :max="max"
                  class="px-1 text-center text-md"
                  @blur="applyHsl"
                  @keyup.enter="applyHsl"
              />
              <InputGroupAddon class="p-1 border-r">
                {{ch.toUpperCase()}}
              </InputGroupAddon>
            </InputGroup>
          </div>
        </div>
      </TabsContent>
    </Tabs>
  </div>
</template>
