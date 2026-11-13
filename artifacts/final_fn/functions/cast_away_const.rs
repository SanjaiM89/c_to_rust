pub fn cast_away_const<'a>(string: &&'a [u8]) -> &'a [u8] {
    *string
}
