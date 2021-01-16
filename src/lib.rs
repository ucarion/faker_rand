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

/// Utility generators that can be used as building blocks for larger
/// generators.
pub mod util {
    /// Generates an ASCII decimal digit (0-9).
    ///
    /// ```
    /// use rand::{Rng, SeedableRng};
    /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
    ///
    /// use faker_rand::util::AsciiDigit;
    /// assert_eq!("7", rng.gen::<AsciiDigit>().to_string());
    /// ```
    pub struct AsciiDigit(String);
    faker_impl_from_file!(AsciiDigit, "data/ascii_digit");

    /// Generates an ASCII lowercase letter (a-z).
    ///
    /// ```
    /// use rand::{Rng, SeedableRng};
    /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
    ///
    /// use faker_rand::util::AsciiLowercase;
    /// assert_eq!("s", rng.gen::<AsciiLowercase>().to_string());
    /// ```
    pub struct AsciiLowercase(String);
    faker_impl_from_file!(AsciiLowercase, "data/ascii_lowercase");

    use rand::distributions::{Distribution, Standard};
    use rand::Rng;
    use std::fmt;
    use std::marker::PhantomData;

    /// Wraps a string generator so that its output is all ASCII lowercase
    /// letters (a-z).
    ///
    /// Any character that can't be lowercased to ASCII lowercase is stripped
    /// from the output.
    ///
    /// ```
    /// use rand::{Rng, SeedableRng};
    /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
    ///
    /// // FirstName generates strings whose first letter is capitalized ...
    /// use faker_rand::en_us::names::FirstName;
    /// assert_eq!("Melvin", rng.gen::<FirstName>().to_string());
    ///
    /// // ... But with ToAsciiLowercase, it's lowercased.
    /// use faker_rand::util::ToAsciiLowercase;
    /// assert_eq!("jamey", rng.gen::<ToAsciiLowercase<FirstName>>().to_string());
    /// ```
    pub struct ToAsciiLowercase<T>(String, PhantomData<T>);

    impl<T: ToString> Distribution<ToAsciiLowercase<T>> for Standard
    where
        Standard: Distribution<T>,
    {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ToAsciiLowercase<T> {
            let mut s = deunicode::deunicode(&rng.gen::<T>().to_string()).to_lowercase();
            s.retain(|c| c.is_ascii_lowercase());

            ToAsciiLowercase(s, std::marker::PhantomData)
        }
    }

    impl<T: ToString> fmt::Display for ToAsciiLowercase<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    /// Wraps a string generator so that the first letter of its output is
    /// capitalized.
    ///
    /// ```
    /// use rand::{Rng, SeedableRng};
    /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
    ///
    /// // Word generates strings that are all lowercase ...
    /// use faker_rand::lorem::Word;
    /// assert_eq!("impedit", rng.gen::<Word>().to_string());
    ///
    /// // ... But with CapitalizeFirstLetter, the first letter is capitalized.
    /// use faker_rand::util::CapitalizeFirstLetter;
    /// assert_eq!("Totam", rng.gen::<CapitalizeFirstLetter<Word>>().to_string());
    /// ```
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

/// Generators for "lorem ipsum" placeholder text.
pub mod lorem {
    use crate::util::CapitalizeFirstLetter;

    /// Generates a lorem ipsum word.
    ///
    /// ```
    /// use rand::{Rng, SeedableRng};
    /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
    ///
    /// use faker_rand::lorem::Word;
    /// assert_eq!("impedit", rng.gen::<Word>().to_string());
    /// ```
    pub struct Word(String);
    faker_impl_from_file!(Word, "data/lorem_words");

    struct FirstWord(String);
    faker_impl_from_templates! {
        FirstWord;

        "{}", CapitalizeFirstLetter<Word>;
    }

    /// Generates a lorem ipsum sentence.
    ///
    /// ```
    /// use rand::{Rng, SeedableRng};
    /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
    ///
    /// use faker_rand::lorem::Sentence;
    /// assert_eq!(
    ///     "Cumque debitis unde eum recusandae aut.",
    ///     rng.gen::<Sentence>().to_string()
    /// );
    /// ```
    pub struct Sentence(String);
    faker_impl_from_templates! {
        Sentence;

        "{} {} {}.", FirstWord, Word, Word;
        "{} {} {} {}.", FirstWord, Word, Word, Word;
        "{} {} {} {} {}.", FirstWord, Word, Word, Word, Word;
        "{} {} {} {} {} {}.", FirstWord, Word, Word, Word, Word, Word;
        "{} {} {} {} {} {} {}.", FirstWord, Word, Word, Word, Word, Word, Word;
    }

    /// Generates a lorem ipsum paragraph.
    ///
    /// ```
    /// use rand::{Rng, SeedableRng};
    /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
    ///
    /// use faker_rand::lorem::Paragraph;
    /// assert_eq!(
    ///     "Debitis unde eum recusandae aut. Aut assumenda cupiditate aliquid voluptas facilis consectetur. Repellendus quae perspiciatis asperiores impedit. Voluptate dolorem in autem et. Consequatur iusto corrupti eum cupiditate.",
    ///     rng.gen::<Paragraph>().to_string()
    /// );
    /// ```
    pub struct Paragraph(String);
    faker_impl_from_templates! {
        Paragraph;

        "{} {} {}", Sentence, Sentence, Sentence;
        "{} {} {} {}", Sentence, Sentence, Sentence, Sentence;
        "{} {} {} {} {}", Sentence, Sentence, Sentence, Sentence, Sentence;
    }

    /// Generates multiple lorem ipsum paragraphs.
    ///
    /// ```
    /// use rand::{Rng, SeedableRng};
    /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
    ///
    /// use faker_rand::lorem::Paragraphs;
    /// assert_eq!(
    ///     "Debitis unde eum recusandae aut. Aut assumenda cupiditate aliquid voluptas facilis consectetur. Repellendus quae perspiciatis asperiores impedit. Voluptate dolorem in autem et. Consequatur iusto corrupti eum cupiditate.\nDignissimos sit cupiditate vitae. Ex quidem odio quia nam. Doloribus reiciendis dignissimos in cum ad reprehenderit.\nEt error illum. Animi voluptatem quo temporibus velit consequatur. Ipsa corrupti cupiditate in et.\nSapiente molestiae sed. Ipsa voluptas rerum laborum. Sed natus et eum officiis ut. Ut voluptatem sint consequatur fuga explicabo asperiores. Aliquam vero quia cupiditate exercitationem blanditiis ea.\nMinima incidunt velit provident voluptate odio. Eius sequi unde voluptas qui. Possimus aut optio et. Consequuntur soluta aut dicta eos amet rerum. Eveniet corporis repudiandae aspernatur.\n",
    ///     rng.gen::<Paragraphs>().to_string()
    /// );
    /// ```
    pub struct Paragraphs(String);
    faker_impl_from_templates! {
        Paragraphs;

        "{}\n{}\n{}\n", Paragraph, Paragraph, Paragraph;
        "{}\n{}\n{}\n{}\n", Paragraph, Paragraph, Paragraph, Paragraph;
        "{}\n{}\n{}\n{}\n{}\n", Paragraph, Paragraph, Paragraph, Paragraph, Paragraph;
    }
}

/// Localized generators for English as spoken in the United States (`en-US`).
pub mod en_us {
    /// Generators for the names of individuals (e.g., first, last, or full
    /// names).
    pub mod names {
        /// Generates a first name.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::names::FirstName;
        /// assert_eq!("Melvin", rng.gen::<FirstName>().to_string());
        /// ```
        pub struct FirstName(String);
        faker_impl_from_file!(FirstName, "data/en_us/first_names");

        /// Generates a last name.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::names::LastName;
        /// assert_eq!("Quitzon", rng.gen::<LastName>().to_string());
        /// ```
        pub struct LastName(String);
        faker_impl_from_file!(LastName, "data/en_us/last_names");

        /// Generates a name prefix.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::names::NamePrefix;
        /// assert_eq!("Miss", rng.gen::<NamePrefix>().to_string());
        /// ```
        pub struct NamePrefix(String);
        faker_impl_from_file!(NamePrefix, "data/en_us/name_prefixes");

        /// Generates a name suffix.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::names::NameSuffix;
        /// assert_eq!("IV", rng.gen::<NameSuffix>().to_string());
        /// ```
        pub struct NameSuffix(String);
        faker_impl_from_file!(NameSuffix, "data/en_us/name_suffixes");

        /// Generates a full name, including possibly a prefix, suffix, or both.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::names::FullName;
        /// assert_eq!("Cleta McClure III", rng.gen::<FullName>().to_string());
        /// ```
        pub struct FullName(String);
        faker_impl_from_templates! {
            FullName;

            "{} {}", FirstName, LastName;
            "{} {} {}", NamePrefix, FirstName, LastName;
            "{} {} {}", FirstName, LastName, NameSuffix;
            "{} {} {} {}", NamePrefix, FirstName, LastName, NameSuffix;
        }
    }

    /// Generators for postal addresses and their constituent parts (e.g. city
    /// names, postal codes, etc.).
    pub mod addresses {
        use super::names::{FirstName, FullName, LastName};
        use crate::util::AsciiDigit;

        struct CityPrefix(String);
        faker_impl_from_file!(CityPrefix, "data/en_us/city_prefixes");

        struct CitySuffix(String);
        faker_impl_from_file!(CitySuffix, "data/en_us/city_suffixes");

        /// Generates a city name.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::addresses::CityName;
        /// assert_eq!("Cletastad", rng.gen::<CityName>().to_string());
        /// ```
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

        /// Generates a street name.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::addresses::StreetName;
        /// assert_eq!("Renner Mission", rng.gen::<StreetName>().to_string());
        /// ```
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

        /// Generates a street address.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::addresses::StreetAddress;
        /// assert_eq!("5489 Shanie Springs", rng.gen::<StreetAddress>().to_string());
        /// ```
        pub struct StreetAddress(String);
        faker_impl_from_templates! {
            StreetAddress;

            "{} {}", BuildingNumber, StreetName;
        }

        /// Generates a secondary address (e.g. an apartment number).
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::addresses::SecondaryAddress;
        /// assert_eq!("Suite 755", rng.gen::<SecondaryAddress>().to_string());
        /// ```
        pub struct SecondaryAddress(String);
        faker_impl_from_templates! {
            SecondaryAddress;

            "Apt. {}{}{}", AsciiDigit, AsciiDigit, AsciiDigit;
            "Suite {}{}{}", AsciiDigit, AsciiDigit, AsciiDigit;
        }

        /// Generates a first-level administrative division (e.g. one of the 50
        /// states).
        ///
        /// Currently, other top-level divisions in USA, such as the District of
        /// Columbia or the unincorporated organized territories (e.g. Puerto
        /// Rico), are not included in this list. This may be changed in a
        /// future minor version of this crate.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::addresses::Division;
        /// assert_eq!("Oklahoma", rng.gen::<Division>().to_string());
        /// ```
        pub struct Division(String);
        faker_impl_from_file!(Division, "data/en_us/divisions");

        /// Generates an abbreviated first-level division (e.g. the two-letter
        /// abbreviation for one of the 50 states).
        ///
        /// See note in [`Division`] on the inclusion of entities other than the
        /// 50 states, and how this may change in a minor version of this crate.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::addresses::DivisionAbbreviation;
        /// assert_eq!("OK", rng.gen::<DivisionAbbreviation>().to_string());
        /// ```
        pub struct DivisionAbbreviation(String);
        faker_impl_from_file!(DivisionAbbreviation, "data/en_us/division_abbreviations");

        /// Generates a postal code (a.k.a. a ZIP Code).
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::addresses::PostalCode;
        /// assert_eq!("75548-9960", rng.gen::<PostalCode>().to_string());
        /// ```
        pub struct PostalCode(String);
        faker_impl_from_templates! {
            PostalCode;

            "{}{}{}{}{}", AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit;
            "{}{}{}{}{}-{}{}{}{}", AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit;
        }

        /// Generates a full postal address.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::addresses::Address;
        /// assert_eq!(
        ///     "Cleta McClure III\n15364 Marks Passage Apt. 057\nMargaritaborough, MA 91404\n",
        ///     rng.gen::<Address>().to_string()
        /// );
        /// ```
        pub struct Address(String);
        faker_impl_from_templates! {
            Address;

            "{}\n{}\n{}, {} {}\n", FullName, StreetAddress, CityName, DivisionAbbreviation, PostalCode;
            "{}\n{} {}\n{}, {} {}\n", FullName, StreetAddress, SecondaryAddress, CityName, DivisionAbbreviation, PostalCode;
        }
    }

    /// Generators for company names and slogans.
    pub mod company {
        use super::names::{FirstName, LastName};

        struct CompanySuffix(String);
        faker_impl_from_file!(CompanySuffix, "data/en_us/company_suffixes");

        /// Generates a company name.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::company::CompanyName;
        /// assert_eq!("Konopelski, Price, and Beier", rng.gen::<CompanyName>().to_string());
        /// ```
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

        /// Generates a company slogan.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::company::Slogan;
        /// assert_eq!("Business-focused intermediate applications", rng.gen::<Slogan>().to_string());
        /// ```
        pub struct Slogan(String);
        faker_impl_from_templates! {
            Slogan;

            "{} {} {}", SloganAdjective, SloganDescriptor, SloganNouns;
        }
    }

    /// Generators for internet domain names, usernames, and emails.
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

        /// Generates a domain name.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::internet::Domain;
        /// assert_eq!("thiel.name", rng.gen::<Domain>().to_string());
        /// ```
        pub struct Domain(String);
        faker_impl_from_templates! {
            Domain;

            "{}.{}", DomainWord, DomainTLD;
        }

        /// Generates a username.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::internet::Username;
        /// assert_eq!("odietrich48", rng.gen::<Username>().to_string());
        /// ```
        pub struct Username(String);
        faker_impl_from_templates! {
            Username;

            "{}{}", AsciiLowercase, ToAsciiLowercase<LastName>;
            "{}{}{}", AsciiLowercase, ToAsciiLowercase<LastName>, AsciiDigit;
            "{}{}{}{}", AsciiLowercase, ToAsciiLowercase<LastName>, AsciiDigit, AsciiDigit;
            "{}{}", ToAsciiLowercase<FirstName>, ToAsciiLowercase<LastName>;
        }

        /// Generates an email.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::internet::Email;
        /// assert_eq!("odietrich48@thompson.net", rng.gen::<Email>().to_string());
        /// ```
        pub struct Email(String);
        faker_impl_from_templates! {
            Email;

            "{}@{}", Username, Domain;
        }
    }

    /// Generators for phone numbers.
    pub mod phones {
        use crate::util::AsciiDigit;

        /// Generates a phone number.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::en_us::phones::PhoneNumber;
        /// assert_eq!("(058) 981-5364", rng.gen::<PhoneNumber>().to_string());
        /// ```
        pub struct PhoneNumber(String);
        faker_impl_from_templates! {
            PhoneNumber;

            "({}{}{}) {}{}{}-{}{}{}{}", AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit;
        }
    }
}

/// Localized generators for French as spoken in France (`fr-FR`).
pub mod fr_fr {
    /// Generators for the names of individuals (e.g., first, last, or full
    /// names).
    pub mod names {
        /// Generates a first name.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::names::FirstName;
        /// assert_eq!("Mahaut", rng.gen::<FirstName>().to_string());
        /// ```
        pub struct FirstName(String);
        faker_impl_from_file!(FirstName, "data/fr_fr/first_names");

        /// Generates a last name.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::names::LastName;
        /// assert_eq!("GUILLOT", rng.gen::<LastName>().to_string());
        /// ```
        pub struct LastName(String);
        faker_impl_from_file!(LastName, "data/fr_fr/last_names");

        /// Generates a name prefix.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::names::NamePrefix;
        /// assert_eq!("Dr", rng.gen::<NamePrefix>().to_string());
        /// ```
        pub struct NamePrefix(String);
        faker_impl_from_file!(NamePrefix, "data/fr_fr/name_prefixes");

        /// Generates a full name, including possibly a prefix.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::names::FullName;
        /// assert_eq!("Mlle Gisèle MARTINEZ", rng.gen::<FullName>().to_string());
        /// ```
        pub struct FullName(String);
        faker_impl_from_templates! {
            FullName;

            "{} {}", FirstName, LastName;
            "{} {} {}", NamePrefix, FirstName, LastName;
        }
    }

    /// Generators for postal addresses and their constituent parts (e.g. city
    /// names, postal codes, etc.).
    pub mod addresses {
        use super::names::FullName;
        use crate::util::AsciiDigit;

        /// Generates a city name.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::addresses::CityName;
        /// assert_eq!("Levallois-Perret", rng.gen::<CityName>().to_string());
        /// ```
        pub struct CityName(String);
        faker_impl_from_file!(CityName, "data/fr_fr/city_names");

        struct StreetPrefix(String);
        faker_impl_from_file!(StreetPrefix, "data/fr_fr/street_prefixes");

        struct StreetSuffix(String);
        faker_impl_from_file!(StreetSuffix, "data/fr_fr/street_suffixes");

        /// Generates a street name.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::addresses::StreetName;
        /// assert_eq!("Passage de Seine", rng.gen::<StreetName>().to_string());
        /// ```
        pub struct StreetName(String);
        faker_impl_from_templates! {
            StreetName;

            "{} {}", StreetPrefix, StreetSuffix;
        }

        struct BuildingNumber(String);
        faker_impl_from_templates! {
            BuildingNumber;

            "{}", AsciiDigit;
            "{}{}", AsciiDigit, AsciiDigit;
            "{}{}{}", AsciiDigit, AsciiDigit, AsciiDigit;
        }

        /// Generates a street address.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::addresses::StreetAddress;
        /// assert_eq!("54 Place de Montmorency", rng.gen::<StreetAddress>().to_string());
        /// ```
        pub struct StreetAddress(String);
        faker_impl_from_templates! {
            StreetAddress;

            "{} {}", BuildingNumber, StreetName;
        }

        /// Generates a secondary address (e.g. an apartment number).
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::addresses::SecondaryAddress;
        /// assert_eq!("7 étage", rng.gen::<SecondaryAddress>().to_string());
        /// ```
        pub struct SecondaryAddress(String);
        faker_impl_from_templates! {
            SecondaryAddress;

            "Apt. {}{}{}", AsciiDigit, AsciiDigit, AsciiDigit;
            "{} étage", AsciiDigit;
        }

        /// Generates a first-level administrative division (e.g. one of the
        /// *régions* of France).
        ///
        /// Currently, this will generate only one of the 13 metropolitan
        /// regions of France. This may be changed in a future minor version of
        /// this crate.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::addresses::Division;
        /// assert_eq!("Nouvelle-Aquitaine", rng.gen::<Division>().to_string());
        /// ```
        pub struct Division(String);
        faker_impl_from_file!(Division, "data/fr_fr/divisions");

        /// Generates a postal code.
        ///
        /// No guarantee is made that the first two digits correspond to a
        /// correct department.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::addresses::PostalCode;
        /// assert_eq!("05898", rng.gen::<PostalCode>().to_string());
        /// ```
        pub struct PostalCode(String);
        faker_impl_from_templates! {
            PostalCode;

            "{}{}{}{}{}", AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit;
        }

        /// Generates a full postal address.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::addresses::Address;
        /// assert_eq!(
        ///     "Mlle Lucille MOREAU\nApt. 489\n96 Quai Saint-Jacques\n05764 Saint-Nazaire\nFRANCE\n",
        ///     rng.gen::<Address>().to_string()
        /// );
        /// ```
        pub struct Address(String);
        faker_impl_from_templates! {
            Address;

            "{}\n{}\n{} {}\nFRANCE\n", FullName, StreetAddress, PostalCode, CityName;
            "{}\n{}\n{}\n{} {}\nFRANCE\n", FullName, SecondaryAddress, StreetAddress, PostalCode, CityName;
        }
    }

    /// Generators for company names.
    pub mod company {
        use super::names::FirstName;

        struct CompanySuffix(String);
        faker_impl_from_file!(CompanySuffix, "data/fr_fr/company_suffixes");

        /// Generates a company name.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::company::CompanyName;
        /// assert_eq!("Lucille SARL", rng.gen::<CompanyName>().to_string());
        /// ```
        pub struct CompanyName(String);
        faker_impl_from_templates! {
            CompanyName;

            "{} {}", FirstName, CompanySuffix;
        }
    }

    /// Generators for internet domain names, usernames, and emails.
    pub mod internet {
        use super::names::{FirstName, LastName};
        use crate::util::{AsciiDigit, AsciiLowercase, ToAsciiLowercase};

        struct DomainWord(String);
        faker_impl_from_templates! {
            DomainWord;

            "{}", ToAsciiLowercase<LastName>;
        }

        struct DomainTLD(String);
        faker_impl_from_file!(DomainTLD, "data/fr_fr/domain_tlds");

        /// Generates a domain name.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::internet::Domain;
        /// assert_eq!("renard.net", rng.gen::<Domain>().to_string());
        /// ```
        pub struct Domain(String);
        faker_impl_from_templates! {
            Domain;

            "{}.{}", DomainWord, DomainTLD;
        }

        /// Generates a username.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::internet::Username;
        /// assert_eq!("omartinez48", rng.gen::<Username>().to_string());
        /// ```
        pub struct Username(String);
        faker_impl_from_templates! {
            Username;

            "{}{}", AsciiLowercase, ToAsciiLowercase<LastName>;
            "{}{}{}", AsciiLowercase, ToAsciiLowercase<LastName>, AsciiDigit;
            "{}{}{}{}", AsciiLowercase, ToAsciiLowercase<LastName>, AsciiDigit, AsciiDigit;
            "{}{}", ToAsciiLowercase<FirstName>, ToAsciiLowercase<LastName>;
        }

        /// Generates an email.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::internet::Email;
        /// assert_eq!("omartinez48@poirier.net", rng.gen::<Email>().to_string());
        /// ```
        pub struct Email(String);
        faker_impl_from_templates! {
            Email;

            "{}@{}", Username, Domain;
        }
    }

    /// Generators for phone numbers.
    pub mod phones {
        use crate::util::AsciiDigit;

        /// Generates a phone number.
        ///
        /// ```
        /// use rand::{Rng, SeedableRng};
        /// let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        ///
        /// use faker_rand::fr_fr::phones::PhoneNumber;
        /// assert_eq!("00 58 98 15 36", rng.gen::<PhoneNumber>().to_string());
        /// ```
        pub struct PhoneNumber(String);
        faker_impl_from_templates! {
            PhoneNumber;

            "0{} {}{} {}{} {}{} {}{}", AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit, AsciiDigit;
        }
    }
}
