//! Profile-related stuff.

use std::{fmt, iter};

/// Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Profile {
    /// Compacted.
    Compacted,
    /// Context.
    Context,
    /// Expanded.
    Expanded,
    /// Flattened.
    Flattened,
    /// Frame.
    Frame,
    /// Framed.
    Framed,
}

impl Profile {
    /// Returns the URI.
    pub fn uri(self) -> &'static str {
        macro_rules! profile_uri {
            ($frag:expr) => {
                concat!("http://www.w3.org/ns/json-ld#", $frag)
            };
        }

        match self {
            Self::Compacted => profile_uri!("compacted"),
            Self::Context => profile_uri!("context"),
            Self::Expanded => profile_uri!("expanded"),
            Self::Flattened => profile_uri!("flattened"),
            Self::Frame => profile_uri!("frame"),
            Self::Framed => profile_uri!("framed"),
        }
    }

    /// Returns an integer with distinct single bit set.
    fn single_bit(self) -> u8 {
        let shift = match self {
            Self::Compacted => 0,
            Self::Context => 1,
            Self::Expanded => 2,
            Self::Flattened => 3,
            Self::Frame => 4,
            Self::Framed => 5,
        };
        1 << shift
    }

    /// Returns an iterator of `Profile` enum variants.
    fn variants() -> impl Iterator<Item = Self> {
        /// List of all variants.
        const ALL_VARIANTS: [Profile; 6] = [
            Profile::Compacted,
            Profile::Context,
            Profile::Expanded,
            Profile::Flattened,
            Profile::Frame,
            Profile::Framed,
        ];
        ALL_VARIANTS.iter().copied()
    }
}

/// Request profile.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestProfile {
    /// A set of profiles as a bitfield.
    profiles: u8,
}

impl RequestProfile {
    /// Creates a new empty `RequestProfile`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks whether the `RequestProfile` contains the given profile.
    pub fn contains(self, profile: Profile) -> bool {
        self.profiles & profile.single_bit() != 0
    }

    /// Returns an iterator of profiles.
    fn iter(self) -> impl Iterator<Item = Profile> {
        Profile::variants().filter(move |v| self.contains(*v))
    }
}

impl fmt::Debug for RequestProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl From<Profile> for RequestProfile {
    fn from(profile: Profile) -> Self {
        Self {
            profiles: profile.single_bit(),
        }
    }
}

impl iter::FromIterator<Profile> for RequestProfile {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Profile>,
    {
        let mut v = RequestProfile::new();
        v.extend(iter.into_iter());
        v
    }
}

impl iter::Extend<Profile> for RequestProfile {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = Profile>,
    {
        iter.into_iter()
            .for_each(|profile| self.profiles |= profile.single_bit());
    }
}
