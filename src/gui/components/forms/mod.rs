//! Переиспользуемые модели полей форм GUI.

pub(crate) mod spi;
pub(crate) mod w5500;

use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::gui::utils::{clamp_index, splice_if_changed};

/// Состояние выпадающего списка формы.
pub(crate) struct ComboField<T> {
    /// Доменные значения, соответствующие строкам GTK-модели.
    pub(crate) items: Vec<T>,
    /// GTK-модель строк.
    pub(crate) model: gtk::StringList,
    /// Текущий выбранный индекс.
    pub(crate) selected_idx: usize,
}

impl<T> ComboField<T> {
    /// Создаёт пустой выпадающий список.
    pub(crate) fn empty() -> Self {
        Self {
            items: Vec::new(),
            model: gtk::StringList::new(&[]),
            selected_idx: 0,
        }
    }

    /// Создаёт выпадающий список с фиксированными вариантами.
    pub(crate) fn new(items: Vec<T>, labels: &[&str]) -> Self {
        Self {
            items,
            model: gtk::StringList::new(labels),
            selected_idx: 0,
        }
    }

    /// Заменяет варианты списка без лишнего GTK-обновления, если строки не изменились.
    pub(crate) fn replace_items(&mut self, items: Vec<T>, labels: &[&str]) {
        self.items = items;
        splice_if_changed(&self.model, labels);
        self.clamp_selected();
    }

    /// Ограничивает выбранный индекс актуальным размером списка.
    pub(crate) fn clamp_selected(&mut self) {
        self.selected_idx = clamp_index(self.selected_idx, self.items.len());
    }

    /// Сбрасывает выбранный индекс.
    pub(crate) fn reset_selected(&mut self, selected_idx: usize) {
        self.selected_idx = clamp_index(selected_idx, self.items.len());
    }

    /// Возвращает `true`, если список пуст.
    pub(crate) fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Возвращает размер списка.
    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }
}

impl<T: Copy> ComboField<T> {
    /// Возвращает выбранный доменный элемент.
    pub(crate) fn selected(&self) -> Option<T> {
        self.items.get(self.selected_idx).copied()
    }
}

/// Состояние текстового поля формы.
pub(crate) struct EntryField {
    /// GTK-буфер поля.
    pub(crate) buffer: gtk::EntryBuffer,
    /// Текущее текстовое значение.
    pub(crate) value: String,
}

impl EntryField {
    /// Создаёт текстовое поле с начальным значением.
    pub(crate) fn new(value: &str) -> Self {
        Self {
            buffer: gtk::EntryBuffer::new(Some(value)),
            value: value.to_string(),
        }
    }

    /// Обновляет локальное значение поля.
    pub(crate) fn set_value(&mut self, value: String) {
        self.value = value;
    }

    /// Обновляет локальное значение и GTK-буфер поля.
    pub(crate) fn set_text(&mut self, value: &str) {
        self.value = value.to_string();
        self.buffer.set_text(value);
    }
}
