using Microsoft.UI;
using System;

namespace WinUIEx
{
    /// <summary>
    /// Information about the ZOrder of the window
    /// </summary>
    /// <seealso cref="WindowEx.ZOrderChanged"/>
    /// <seealso cref="WindowEx.OnZOrderChanged"/>
    public struct ZOrderInfo
    {
        /// <summary>
        /// Gets a value indicating whether the window's Z Order is at the top.
        /// </summary>
        /// <seealso cref="Microsoft.UI.Windowing.AppWindowChangedEventArgs.IsZOrderAtTop"/>
        public bool IsZOrderAtTop { get; init; }

        /// <summary>
        /// Gets a value indicating whether the window's Z Order is at the bottom.
        /// </summary>
        /// <seealso cref="Microsoft.UI.Windowing.AppWindowChangedEventArgs.IsZOrderAtBottom"/>
        public bool IsZOrderAtBottom { get; init; }

        /// <summary>
        /// Gets the id of the window this window is below.
        /// </summary>
        /// <seealso cref="Microsoft.UI.Windowing.AppWindowChangedEventArgs.ZOrderBelowWindowId"/>
        public WindowId ZOrderBelowWindowId { get; init; }
    }
}
