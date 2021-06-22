make_add_macro! {
    name: nom_add_as_bytes;
    type: ::nom::AsBytes;
    body: {
        #[inline]
        fn as_bytes(&self) -> &[u8] {
            self.as_ref()
        }
    }
}

make_add_macro! {
    name: nom_add_input_len;
    type: ::nom::InputLength;
    body: {
        #[inline]
        fn input_len(&self) -> usize {
            ::nom::InputLength::input_len(&self.as_ref())
        }
    }
}

make_add_macro! {
    name: nom_add_input_take;
    type: ::nom::InputTake;
    body: {
        fn take(&self, count: usize) -> Self {
            Self::from_slice_with_offset(
                ::nom::InputTake::take(&self.as_ref(), count),
                self.initial_offset(),
                self.endidness()
            ).unwrap()
        }

        fn take_split(&self, count: usize) -> (Self, Self) {
            let (s1, s2) = ::nom::InputTake::take_split(&self.as_ref(), count);
            (
                Self::from_slice_with_offset(s1, self.initial_offset(), self.endidness()).unwrap(),
                Self::from_slice_with_offset(s2, self.initial_offset() + count,
                    self.endidness()).unwrap()
            )
        }
    }
}

make_add_macro! {
    name: nom_add_offset;
    type: ::nom::Offset;
    body: {
        #[inline]
        fn offset(&self, other: &Self) -> usize {
            ::nom::Offset::offset(&self.as_ref(), &other.as_ref())
        }
    }
}

make_add_macro! {
    name: nom_add_hex_display;
    type: ::nom::HexDisplay;
    body: {
        #[inline]
        fn to_hex(&self, chunk_size: usize) -> String {
            ::nom::HexDisplay::to_hex(self.as_ref(), chunk_size)
        }

        #[inline]
        fn to_hex_from(&self, chunk_size: usize, from: usize) -> String {
            ::nom::HexDisplay::to_hex_from(self.as_ref(), chunk_size, from)
        }
    }
}

#[macro_export]
macro_rules! nom_add_find_substring {
    ($t:path) => {
        impl<'o> ::nom::FindSubstring<&'o [u8]> for $t {
            #[inline]
            fn find_substring(&self, substr: &'o [u8]) -> Option<usize> {
                ::nom::FindSubstring::find_substring(&self.as_ref(), substr)
            }
        }
        impl<'o> ::nom::FindSubstring<&'o str> for $t {
            #[inline]
            fn find_substring(&self, substr: &'o str) -> Option<usize> {
                ::nom::FindSubstring::find_substring(&self.as_ref(), substr)
            }
        }
    };

    ($t:path, $($l:lifetime),+ $($i:ident),*) => {
        impl<'sstr, $($l,)+ $($i,)*> ::nom::FindSubstring<&'sstr [u8]> for $t
        where
            $($l : 'sstr,)+
        {
            #[inline]
            fn find_substring(&self, substr: &'sstr [u8]) -> Option<usize> {
                ::nom::FindSubstring::find_substring(&self.as_ref(), substr)
            }
        }

        impl<'sstr, $($l,)+ $($i,)*> ::nom::FindSubstring<&'sstr str> for $t
        where
            $($l : 'sstr,)+
        {
            #[inline]
            fn find_substring(&self, substr: &'sstr str) -> Option<usize> {
                ::nom::FindSubstring::find_substring(&self.as_ref(), substr)
            }
        }
    }
}

#[macro_export]
macro_rules! nom_add_find_token {
    ($t:path) => {
        impl ::nom::FindToken<u8> for $t {
            #[inline]
            fn find_token(&self, token: u8) -> bool {
                ::nom::FindToken::find_token(&self.as_ref(), token)
            }
        }

        impl<'t> ::nom::FindToken<&'t u8> for $t {
            #[inline]
            fn find_token(&self, token: &'t u8) -> bool {
                ::nom::FindToken::find_token(&self.as_ref(), token)
            }
        }

        impl ::nom::FindToken<char> for $t {
            #[inline]
            fn find_token(&self, token: char) -> bool {
                ::nom::FindToken::find_token(&self.as_ref(), token)
            }
        }
    };
    ($t:path, $($l:lifetime),+ $($i:ident),*) => {
        impl<$($l,)+ $($i,)*> ::nom::FindToken<u8> for $t {
            fn find_token(&self, token: u8) -> bool {
                ::nom::FindToken::find_token(&self.as_ref(), token)
            }
        }
        impl<'token, $($l,)+ $($i,)*> ::nom::FindToken<&'token u8> for $t
        where
            $($l : 'token,)+
        {
            fn find_token(&self, token: &'token u8) -> bool {
                ::nom::FindToken::find_token(&self.as_ref(), token)
            }
        }
        impl<$($l,)+ $($i,)*> ::nom::FindToken<char> for $t {
            fn find_token(&self, token: char) -> bool {
                ::nom::FindToken::find_token(&self.as_ref(), token)
            }
        }
    }
}

#[macro_export]
macro_rules! add_all_noms {
    ($t:path) => {
        nom_add_as_bytes! { $t }
        nom_add_input_len! { $t }
        nom_add_input_take! { $t }
        nom_add_offset! { $t }
        nom_add_hex_display! { $t }
        nom_add_find_substring! { $t }
        nom_add_find_token! { $t }
    };
    ($t:path, $($l:lifetime),+ $($i:ident),*) => {
        nom_add_as_bytes! { $t, $($l),+ $($i),* }
        nom_add_input_len! { $t, $($l),+ $($i),* }
        nom_add_input_take! { $t, $($l),+ $($i),* }
        nom_add_offset! { $t, $($l),+ $($i),* }
        nom_add_hex_display! { $t, $($l),+ $($i),* }
        nom_add_find_substring! { $t, $($l),+ $($i),* }
        nom_add_find_token! { $t, $($l),+ $($i),* }
    }
}
