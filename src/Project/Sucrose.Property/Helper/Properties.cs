using SMMM = Sucrose.Manager.Manage.Manager;
using SPMI = Sucrose.Property.Manage.Internal;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSTHP = Sucrose.Shared.Theme.Helper.Properties;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;

namespace Sucrose.Property.Helper
{
    internal static class Properties
    {
        public static void Change(string Key, SSTMCM Data)
        {
            if (SMMM.LibrarySelected == SPMI.Library && SSSHL.Run())
            {
                SSTHP.WriteJson(SPMI.WatcherFile.Replace("*", $"{Guid.NewGuid()}"), new()
                {
                    PropertyListener = SPMI.Properties.PropertyListener,
                    PropertyList = new()
                    {
                        [Key] = Data
                    }
                });
            }

            SPMI.Properties.PropertyList[Key] = Data;

            SSTHP.WriteJson(SPMI.PropertiesFile, SPMI.Properties);
        }
    }
}