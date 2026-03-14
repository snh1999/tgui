export interface IInputProps {
  modelValue?: string;
  id?: string;
  name?: string;
  placeholder?: string;
  disabled?: boolean;
}

export interface IInputEmits {
  "update:modelValue": [value: string];
}
