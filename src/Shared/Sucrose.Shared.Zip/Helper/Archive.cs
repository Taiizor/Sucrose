using System.IO;
using SHV = Skylark.Helper.Versionly;
using SMMRC = Sucrose.Memory.Manage.Readonly.Content;
using SSDECT = Sucrose.Shared.Dependency.Enum.CompatibilityType;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSSHA = Sucrose.Shared.Space.Helper.Access;
using SSTHC = Sucrose.Shared.Theme.Helper.Compatible;
using SSTHF = Sucrose.Shared.Theme.Helper.Filter;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHP = Sucrose.Shared.Theme.Helper.Properties;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using SSZHZ = Sucrose.Shared.Zip.Helper.Zip;

namespace Sucrose.Shared.Zip.Helper
{
    internal static class Archive
    {
        public static SSDECT Check(string Archive)
        {
            try
            {
                if (!File.Exists(Archive))
                {
                    return SSDECT.NotFound;
                }

                if (!SSSHA.File(Archive))
                {
                    return SSDECT.Access;
                }

                if (Path.GetExtension(Archive) != ".zip")
                {
                    return SSDECT.Extension;
                }

                if (!SSZHZ.CheckArchive(Archive))
                {
                    return SSDECT.ZipType;
                }

                //if (SSZHZ.EncryptedArchive(Archive))
                //{
                //    return SSDECT.Encrypt;
                //}

                if (!SSZHZ.CheckFolder(Archive))
                {
                    return SSDECT.Folder;
                }

                if (!SSZHZ.CheckFile(Archive, SMMRC.SucroseInfo))
                {
                    return SSDECT.InfoFile;
                }

                string Salt = SSZHZ.ReadFile(Archive, SMMRC.SucroseInfo);

                if (string.IsNullOrEmpty(Salt))
                {
                    return SSDECT.EmptyInfo;
                }

                bool Json = SSTHI.FromCheck(Salt);

                if (!Json)
                {
                    return SSDECT.InvalidInfo;
                }

                SSTHI Info = SSTHI.FromJson(Salt);

                if (string.IsNullOrEmpty(Info.Title) || Info.Title.Length > 50)
                {
                    return SSDECT.Title;
                }

                if (string.IsNullOrEmpty(Info.Description) || Info.Description.Length > 500)
                {
                    return SSDECT.Description;
                }

                if (string.IsNullOrEmpty(Info.Author) || Info.Author.Length > 50)
                {
                    return SSDECT.Author;
                }

                if (string.IsNullOrEmpty(Info.Contact) || Info.Contact.Length > 250)
                {
                    return SSDECT.Contact;
                }

                if (!SSTHV.IsUrl(Info.Contact) && !SSTHV.IsMail(Info.Contact))
                {
                    return SSDECT.InvalidContact;
                }

                if (Info.Tags != null && Info.Tags.Any() && (Info.Tags.Count() < 1 || Info.Tags.Count() > 5))
                {
                    return SSDECT.Tags;
                }

                if (Info.Tags != null && Info.Tags.Any() && Info.Tags.Any(Tag => Tag.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(Tag)))
                {
                    return SSDECT.InvalidTags;
                }

                if (!string.IsNullOrEmpty(Info.Arguments) && Info.Arguments.Length > 250)
                {
                    return SSDECT.Arguments;
                }

                if (!SSZHZ.CheckFile(Archive, Info.Thumbnail))
                {
                    return SSDECT.Thumbnail;
                }

                if (!SSZHZ.CheckFile(Archive, Info.Preview))
                {
                    return SSDECT.Preview;
                }

                if (Info.AppVersion.CompareTo(SHV.Entry()) > 0)
                {
                    return SSDECT.AppVersion;
                }

                if ((int)Info.Type >= Enum.GetValues(typeof(SSDEWT)).Length)
                {
                    return SSDECT.Type;
                }

                if (Info.Type == SSDEWT.Gif)
                {
                    if (!SSTHV.IsUrl(Info.Source) && !SSZHZ.CheckFile(Archive, Info.Source))
                    {
                        return SSDECT.Source;
                    }
                    else if (!SSTHF.GifExtension(Info.Source))
                    {
                        return SSDECT.InvalidExtension;
                    }
                }
                else if (Info.Type == SSDEWT.Url && !SSTHV.IsUrl(Info.Source))
                {
                    return SSDECT.InvalidUrl;
                }
                else if (Info.Type == SSDEWT.Web)
                {
                    if (!SSZHZ.CheckFile(Archive, Info.Source))
                    {
                        return SSDECT.Source;
                    }
                    else if (!SSTHF.WebExtension(Info.Source))
                    {
                        return SSDECT.InvalidExtension;
                    }
                }
                else if (Info.Type == SSDEWT.Video)
                {
                    if (!SSTHV.IsUrl(Info.Source) && !SSZHZ.CheckFile(Archive, Info.Source))
                    {
                        return SSDECT.Source;
                    }
                    else if (!SSTHF.VideoExtension(Info.Source))
                    {
                        return SSDECT.InvalidExtension;
                    }
                }
                else if (Info.Type == SSDEWT.YouTube && !SSTHV.IsYouTubeAll(Info.Source))
                {
                    return SSDECT.InvalidUrl;
                }
                else if (Info.Type == SSDEWT.Application)
                {
                    if (!SSZHZ.CheckFile(Archive, Info.Source))
                    {
                        return SSDECT.Source;
                    }
                    else if (!SSTHF.AppExtension(Info.Source))
                    {
                        return SSDECT.InvalidExtension;
                    }
                }

                if (SSZHZ.CheckFile(Archive, SMMRC.SucroseProperties))
                {
                    Salt = SSZHZ.ReadFile(Archive, SMMRC.SucroseProperties);

                    if (string.IsNullOrEmpty(Salt))
                    {
                        return SSDECT.EmptyProperties;
                    }

                    Json = SSTHP.FromCheck(Salt);

                    if (!Json)
                    {
                        return SSDECT.InvalidProperties;
                    }

                    SSTHP Properties = SSTHP.FromJson(Salt);

                    if (!string.IsNullOrEmpty(Properties.PropertyListener) && (!Properties.PropertyListener.Contains("{0}") || !Properties.PropertyListener.Contains("{1}")))
                    {
                        return SSDECT.PropertyListener;
                    }

                    if (Properties.PropertyList == null || !Properties.PropertyList.Any())
                    {
                        return SSDECT.PropertyList;
                    }

                    if (Properties.PropertyLocalization != null && !Properties.PropertyLocalization.Any())
                    {
                        return SSDECT.PropertyLocalization;
                    }
                }

                if (SSZHZ.CheckFile(Archive, SMMRC.SucroseCompatible))
                {
                    Salt = SSZHZ.ReadFile(Archive, SMMRC.SucroseCompatible);

                    if (string.IsNullOrEmpty(Salt))
                    {
                        return SSDECT.EmptyCompatible;
                    }

                    Json = SSTHC.FromCheck(Salt);

                    if (!Json)
                    {
                        return SSDECT.InvalidCompatible;
                    }

                    SSTHC Compatible = SSTHC.FromJson(Salt);

                    if (!string.IsNullOrEmpty(Compatible.LoopMode) && !Compatible.LoopMode.Contains("{0}"))
                    {
                        return SSDECT.LoopMode;
                    }

                    if (!string.IsNullOrEmpty(Compatible.SystemCpu) && !Compatible.SystemCpu.Contains("{0}"))
                    {
                        return SSDECT.SystemCpu;
                    }

                    if (!string.IsNullOrEmpty(Compatible.ThemeType) && !Compatible.ThemeType.Contains("{0}"))
                    {
                        return SSDECT.ThemeType;
                    }

                    if (!string.IsNullOrEmpty(Compatible.SystemBios) && !Compatible.SystemBios.Contains("{0}"))
                    {
                        return SSDECT.SystemBios;
                    }

                    if (!string.IsNullOrEmpty(Compatible.SystemDate) && !Compatible.SystemDate.Contains("{0}"))
                    {
                        return SSDECT.SystemDate;
                    }

                    if (!string.IsNullOrEmpty(Compatible.ShuffleMode) && !Compatible.ShuffleMode.Contains("{0}"))
                    {
                        return SSDECT.ShuffleMode;
                    }

                    if (!string.IsNullOrEmpty(Compatible.StretchMode) && !Compatible.StretchMode.Contains("{0}"))
                    {
                        return SSDECT.StretchMode;
                    }

                    if (!string.IsNullOrEmpty(Compatible.SystemAudio) && !Compatible.SystemAudio.Contains("{0}"))
                    {
                        return SSDECT.SystemAudio;
                    }

                    if (!string.IsNullOrEmpty(Compatible.SystemTheme) && !Compatible.SystemTheme.Contains("{0}"))
                    {
                        return SSDECT.SystemTheme;
                    }

                    if (!string.IsNullOrEmpty(Compatible.VolumeLevel) && !Compatible.VolumeLevel.Contains("{0}"))
                    {
                        return SSDECT.VolumeLevel;
                    }

                    if (!string.IsNullOrEmpty(Compatible.SystemMemory) && !Compatible.SystemMemory.Contains("{0}"))
                    {
                        return SSDECT.SystemMemory;
                    }

                    if (!string.IsNullOrEmpty(Compatible.SystemBattery) && !Compatible.SystemBattery.Contains("{0}"))
                    {
                        return SSDECT.SystemBattery;
                    }

                    if (!string.IsNullOrEmpty(Compatible.SystemGraphic) && !Compatible.SystemGraphic.Contains("{0}"))
                    {
                        return SSDECT.SystemGraphic;
                    }

                    if (!string.IsNullOrEmpty(Compatible.SystemNetwork) && !Compatible.SystemNetwork.Contains("{0}"))
                    {
                        return SSDECT.SystemNetwork;
                    }

                    if (!string.IsNullOrEmpty(Compatible.SystemMotherboard) && !Compatible.SystemMotherboard.Contains("{0}"))
                    {
                        return SSDECT.SystemMotherboard;
                    }
                }

                return SSDECT.Pass;
            }
            catch
            {
                return SSDECT.UnforeseenConsequences;
            }
        }
    }
}