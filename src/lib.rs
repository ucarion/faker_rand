#[macro_export]
macro_rules! faker_impl_from_file {
    ($name: ident, $file: expr) => {
        impl rand::distributions::Distribution<$name> for rand::distributions::Standard {
            fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> $name {
                use lazy_static::lazy_static;

                lazy_static! {
                    static ref VALUES: Vec<String> =
                        include_str!($file).lines().map(String::from).collect();
                }

                $name(VALUES[rng.gen_range(0..VALUES.len())].clone())
            }
        }

        display_impl_for_wrapper!($name);
    };
}

#[macro_export]
macro_rules! faker_impl_from_templates {
    ($name: ident; $($fmt: expr, $($arg:ty),+);+;) => {
        impl rand::distributions::Distribution<$name> for rand::distributions::Standard {
            fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> $name {
                use rand::seq::SliceRandom;
                let funcs: Vec<Box<dyn Fn(&mut R) -> String>> = vec![
                    $(
                        Box::new(|rng| {
                            format!($fmt, $(
                                rng.gen::<$arg>().to_string(),
                            )*)
                        }),
                    )*
                ];

                $name(funcs.choose(rng).unwrap()(rng))
            }
        }

        display_impl_for_wrapper!($name);
    }
}

macro_rules! display_impl_for_wrapper {
    ($name: ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

pub mod util {
    pub struct AsciiDigit(String);
    faker_impl_from_file!(AsciiDigit, "data/ascii_digit");

    pub struct AsciiLowercase(String);
    faker_impl_from_file!(AsciiLowercase, "data/ascii_lowercase");

    use rand::distributions::{Distribution, Standard};
    use rand::Rng;
    use std::fmt;
    use std::marker::PhantomData;

    pub struct ToAsciiLowercase<T>(String, PhantomData<T>);

    impl<T: ToString> Distribution<ToAsciiLowercase<T>> for Standard
    where
        Standard: Distribution<T>,
    {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ToAsciiLowercase<T> {
            let mut s = rng.gen::<T>().to_string().to_lowercase();
            s.retain(|c| c.is_ascii_lowercase());

            ToAsciiLowercase(s, std::marker::PhantomData)
        }
    }

    impl<T: ToString> fmt::Display for ToAsciiLowercase<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    pub struct CapitalizeFirstLetter<T>(String, PhantomData<T>);

    impl<T: ToString> Distribution<CapitalizeFirstLetter<T>> for Standard
    where
        Standard: Distribution<T>,
    {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CapitalizeFirstLetter<T> {
            let s = rng.gen::<T>().to_string();
            let mut c = s.chars();

            CapitalizeFirstLetter(
                c.next().unwrap().to_uppercase().chain(c).collect(),
                std::marker::PhantomData,
            )
        }
    }

    impl<T: ToString> fmt::Display for CapitalizeFirstLetter<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
}

pub mod lorem {
    use crate::util::CapitalizeFirstLetter;

    pub struct Word(String);
    faker_impl_from_file!(Word, "data/lorem_words");

    struct FirstWord(String);
    faker_impl_from_templates! {
        FirstWord;

        "{}", CapitalizeFirstLetter<Word>;
    }

    pub struct Sentence(String);
    faker_impl_from_templates! {
        Sentence;

        "{} {} {}.", FirstWord, Word, Word;
        "{} {} {} {}.", FirstWord, Word, Word, Word;
        "{} {} {} {} {}.", FirstWord, Word, Word, Word, Word;
        "{} {} {} {} {} {}.", FirstWord, Word, Word, Word, Word, Word;
        "{} {} {} {} {} {} {}.", FirstWord, Word, Word, Word, Word, Word, Word;
    }

    pub struct Paragraph(String);
    faker_impl_from_templates! {
        Paragraph;

        "{} {} {}", Sentence, Sentence, Sentence;
        "{} {} {} {}", Sentence, Sentence, Sentence, Sentence;
        "{} {} {} {} {}", Sentence, Sentence, Sentence, Sentence, Sentence;
    }

    pub struct Paragraphs(String);
    faker_impl_from_templates! {
        Paragraphs;

        "{}\n{}\n{}\n", Paragraph, Paragraph, Paragraph;
        "{}\n{}\n{}\n{}\n", Paragraph, Paragraph, Paragraph, Paragraph;
        "{}\n{}\n{}\n{}\n{}\n", Paragraph, Paragraph, Paragraph, Paragraph, Paragraph;
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn sample_name() {
            for _ in 0..100 {
                println!("{}", rand::random::<super::Paragraphs>().to_string());
            }

            panic!();
        }
    }
}

pub mod en_us {
    pub mod names {
        pub struct FirstName(String);
        faker_impl_from_file!(FirstName, "data/en_us/first_names");

        pub struct LastName(String);
        faker_impl_from_file!(LastName, "data/en_us/last_names");

        pub struct NamePrefix(String);
        faker_impl_from_file!(NamePrefix, "data/en_us/name_prefixes");

        pub struct NameSuffix(String);
        faker_impl_from_file!(NameSuffix, "data/en_us/name_suffixes");

        /// A full name.
        pub struct FullName(String);
        faker_impl_from_templates! {
            FullName;

            "{} {}", FirstName, LastName;
            "{} {} {}", NamePrefix, FirstName, LastName;
            "{} {} {}", FirstName, LastName, NameSuffix;
            "{} {} {} {}", NamePrefix, FirstName, LastName, NameSuffix;
        }
    }

    pub mod addresses {
        use super::names::{FirstName, FullName, LastName};
        use crate::util::AsciiDigit;

        struct CityPrefix(String);
        faker_impl_from_file!(CityPrefix, "data/en_us/city_prefixes");

        struct CitySuffix(String);
        faker_impl_from_file!(CitySuffix, "data/en_us/city_suffixes");

        pub struct CityName(String);
        faker_impl_from_templates! {
            CityName;

            "{} {}{}", CityPrefix, FirstName, CitySuffix;
            "{} {}", CityPrefix, FirstName;
            "{}{}", FirstName, CitySuffix;
            "{}{}", LastName, CitySuffix;
        }

        struct StreetSuffix(String);
        faker_impl_from_file!(StreetSuffix, "data/en_us/street_suffixes");

        pub struct StreetName(String);
        faker_impl_from_templates! {
            StreetName;

            "{} {}", FirstName, StreetSuffix;
            "{} {}", LastName, StreetSuffix;
        }

        struct BuildingNumber(String);
        faker_impl_from_templates! {
            BuildingNumber;

            "{}{}{}", AsciiDigit, AsciiDigit, AsciiDigit;
            "{}{}{}{}", AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit;
            "{}{}{}{}{}", AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit;
        }

        pub struct StreetAddress(String);
        faker_impl_from_templates! {
            StreetAddress;

            "{} {}", BuildingNumber, StreetName;
        }

        pub struct SecondaryAddress(String);
        faker_impl_from_templates! {
            SecondaryAddress;

            "Apt. {}{}{}", AsciiDigit, AsciiDigit, AsciiDigit;
            "Suite {}{}{}", AsciiDigit, AsciiDigit, AsciiDigit;
        }

        pub struct Division(String);
        faker_impl_from_file!(Division, "data/en_us/divisions");

        pub struct DivisionAbbreviation(String);
        faker_impl_from_file!(DivisionAbbreviation, "data/en_us/division_abbreviations");

        pub struct PostalCode(String);
        faker_impl_from_templates! {
            PostalCode;

            "{}{}{}{}{}", AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit;
            "{}{}{}{}{}-{}{}{}{}", AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit;
        }

        pub struct Address(String);
        faker_impl_from_templates! {
            Address;

            "{}\n{}\n{}, {} {}\n", FullName, StreetAddress, CityName, DivisionAbbreviation, PostalCode;
            "{}\n{} {}\n{}, {} {}\n", FullName, StreetAddress, SecondaryAddress, CityName, DivisionAbbreviation, PostalCode;
        }
    }

    pub mod company {
        use super::names::{FirstName, LastName};

        struct CompanySuffix(String);
        faker_impl_from_file!(CompanySuffix, "data/en_us/company_suffixes");

        pub struct CompanyName(String);
        faker_impl_from_templates! {
            CompanyName;

            "{} {}", FirstName, CompanySuffix;
            "{}-{}", LastName, LastName;
            "{}, {}, and {}", LastName, LastName, LastName;
        }

        struct SloganAdjective(String);
        faker_impl_from_file!(SloganAdjective, "data/en_us/slogan_adjectives");

        struct SloganDescriptor(String);
        faker_impl_from_file!(SloganDescriptor, "data/en_us/slogan_descriptors");

        struct SloganNouns(String);
        faker_impl_from_file!(SloganNouns, "data/en_us/slogan_nouns");

        pub struct Slogan(String);
        faker_impl_from_templates! {
            Slogan;

            "{} {} {}", SloganAdjective, SloganDescriptor, SloganNouns;
        }
    }

    pub mod internet {
        use super::names::{FirstName, LastName};
        use crate::util::{AsciiDigit, AsciiLowercase, ToAsciiLowercase};

        struct DomainWord(String);
        faker_impl_from_templates! {
            DomainWord;

            "{}", ToAsciiLowercase<LastName>;
        }

        struct DomainTLD(String);
        faker_impl_from_file!(DomainTLD, "data/en_us/domain_tlds");

        pub struct Domain(String);
        faker_impl_from_templates! {
            Domain;

            "{}.{}", DomainWord, DomainTLD;
        }

        pub struct Username(String);
        faker_impl_from_templates! {
            Username;

            "{}{}", AsciiLowercase, ToAsciiLowercase<LastName>;
            "{}{}{}", AsciiLowercase, ToAsciiLowercase<LastName>, AsciiDigit;
            "{}{}{}{}", AsciiLowercase, ToAsciiLowercase<LastName>, AsciiDigit, AsciiDigit;
            "{}{}", ToAsciiLowercase<FirstName>, ToAsciiLowercase<LastName>;
        }

        pub struct Email(String);
        faker_impl_from_templates! {
            Email;

            "{}@{}", Username, Domain;
        }
    }

    pub mod phones {
        use crate::util::AsciiDigit;

        pub struct PhoneNumber(String);
        faker_impl_from_templates! {
            PhoneNumber;

            "{}{}{}-{}{}{}-{}{}{}{}", AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit;
        }
    }
}
