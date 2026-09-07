// Machine-readable profile adapter shared by the CV and cover letter.
#let profile-path = sys.inputs.at("profile", default: "/cvl/profile.toml")
#let profile-data = toml(profile-path)
#assert(profile-data.schema_version == 1, message: "unsupported profile schema version")

#let profile = (
  name: profile-data.name,
  email: profile-data.email,
  phone-label: profile-data.phone_label,
  phone-href: profile-data.phone_href,
  location: profile-data.location,
  languages: profile-data.languages,
  linkedin: profile-data.linkedin,
  website: profile-data.website,
)

#let localized-profile = (
  "de-ch": (
    nationality-and-permit: profile-data.localized.at("de-ch").nationality_and_permit,
    availability: profile-data.localized.at("de-ch").availability,
  ),
  "en-ch": (
    nationality-and-permit: profile-data.localized.at("en-ch").nationality_and_permit,
    availability: profile-data.localized.at("en-ch").availability,
  ),
)
