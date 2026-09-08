// Machine-readable profile adapter shared by the CV and cover letter.
#let load-profile(profile-path) = {
  let profile-data = toml(profile-path)
  assert(profile-data.schema_version == 1, message: "unsupported profile schema version")

  let profile = (
    name: profile-data.name,
    email: profile-data.email,
    phone-label: profile-data.phone_label,
    phone-href: profile-data.phone_href,
    location: profile-data.location,
    languages: profile-data.languages,
    linkedin: profile-data.linkedin,
    website: profile-data.website,
  )

  let localized-profile = (:)
  for (locale, fields) in profile-data.localized {
    localized-profile.insert(locale, (
      nationality-and-permit: fields.nationality_and_permit,
      availability: fields.availability,
    ))
  }

  (profile: profile, localized-profile: localized-profile)
}
