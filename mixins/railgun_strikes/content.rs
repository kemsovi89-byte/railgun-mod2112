//! ТОЧКА ИНТЕГРАЦИИ С КОНТЕНТОМ.
//! Предмет railgun:railgun_designator объявляется контент-модом (дата-модом):
//! там же задаются модель, текстура и рецепт. Этот файл лишь отвечает на
//! вопрос «пушка ли в руке».

pub fn is_railgun_held(_p: &crate::game::Player<'_>) -> bool {
    // TODO(api): сравнить held item id с "railgun:railgun_designator".
    false
}
