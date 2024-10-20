﻿using SMML = Sucrose.Manager.Manage.Library;
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
            if (SMML.Selected == SPMI.LibrarySelected && SSSHL.Run())
            {
                SSTHP.Write(SPMI.WatcherFile.Replace("*", $"{Guid.NewGuid()}"), new()
                {
                    PropertyListener = SPMI.Properties.PropertyListener,
                    PropertyList = new()
                    {
                        [Key] = Data
                    }
                });
            }

            SPMI.Properties.PropertyList[Key] = Data;

            SSTHP.Write(SPMI.PropertiesFile, SPMI.Properties);
        }
    }
}