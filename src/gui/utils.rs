use crate::core::config::SpiMode;
use gtk::prelude::*;
use relm4::gtk;

/// Обновляет [`gtk::StringList`] только при реальном изменении содержимого.
///
/// `StringList::splice` может сбрасывать выбранный элемент и генерировать
/// `notify::selected`, поэтому вызывающий код должен сам защититься от
/// echo-событий, если список связан с реактивным `ComboRow`.
pub(crate) fn splice_if_changed(model: &gtk::StringList, new_values: &[&str]) {
    let current_len = model.n_items();
    let mut changed = current_len as usize != new_values.len();

    if !changed {
        for i in 0..current_len {
            if let Some(item) = model.item(i)
                && let Ok(string_obj) = item.downcast::<gtk::StringObject>()
                && string_obj.string() != new_values[i as usize]
            {
                changed = true;
                break;
            }
        }
    }

    if changed {
        model.splice(0, current_len, new_values);
    }
}

/// Возвращает `idx`, если он входит в диапазон `0..len`, иначе последний валидный индекс.
///
/// Для пустого списка возвращает `0`, потому что GTK `ComboRow` всё равно
/// хранит выбранный индекс как число, а фактическая доступность обычно
/// контролируется через `set_sensitive`.
pub(crate) fn clamp_index(idx: usize, len: usize) -> usize {
    if len == 0 { 0 } else { idx.min(len - 1) }
}

/// Возвращает предпочитаемый индекс, если он существует.
///
/// Используется для стартового выбора разных значений в соседних списках без
/// динамического вырезания выбранных элементов из других `ComboRow`.
pub(crate) fn default_distinct_pin_index(preferred_idx: usize, len: usize) -> usize {
    if len > preferred_idx {
        preferred_idx
    } else {
        0
    }
}

/// Конвертирует индекс из `ComboRow` в [`SpiMode`].
///
/// Использует `strum::FromRepr`, сгенерированный для core-enum. Невалидный
/// индекс трактуется как значение [`SpiMode::default`].
pub(crate) fn mode_from_index(idx: usize) -> SpiMode {
    SpiMode::from_repr(idx as u8).unwrap_or_default()
}
