

namespace Windows.Win32
{
    using global::System;
    using global::System.Diagnostics;
    using global::System.Runtime.CompilerServices;
    using global::System.Runtime.InteropServices;
    using global::Windows.Win32;
    using global::Windows.Win32.Foundation;
    using global::Windows.Win32.UI.WindowsAndMessaging;

    /// <content>
    /// Contains extern methods from "Shell32.dll".
    /// </content>
    internal static partial class PInvoke
        {
        /// <inheritdoc cref = "SHAppBarMessage(uint, APPBARDATA32*)"/>
        internal static unsafe nuint SHAppBarMessage(uint dwMessage, ref APPBARDATA32 pData)
        {
            fixed (APPBARDATA32* pDataLocal = &pData)
            {
                nuint __result = PInvoke.SHAppBarMessage(dwMessage, pDataLocal);
                return __result;
            }
        }
        /// <inheritdoc cref = "SHAppBarMessage(uint, APPBARDATA64*)"/>
        internal static unsafe nuint SHAppBarMessage(uint dwMessage, ref APPBARDATA64 pData)
        {
            fixed (APPBARDATA64* pDataLocal = &pData)
            {
                nuint __result = PInvoke.SHAppBarMessage(dwMessage, pDataLocal);
                return __result;
            }
        }

        /// <summary>Sends an appbar message to the system.</summary>
        /// <param name="dwMessage">Type: <b>DWORD</b></param>
        /// <param name="pData">
        /// <para>Type: <b>PAPPBARDATA</b></para>
        /// <para>A pointer to an <a href="https://docs.microsoft.com/windows/desktop/api/shellapi/ns-shellapi-appbardata">APPBARDATA</a> structure. The content of the structure on entry and on exit depends on the value set in the <i>dwMessage</i> parameter. See the individual message pages for specifics.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/nf-shellapi-shappbarmessage#parameters">Read more on docs.microsoft.com</see>.</para>
        /// </param>
        /// <returns>
        /// <para>Type: <b>UINT_PTR</b></para>
        /// <para>This function returns a message-dependent value. For more information, see the Windows SDK documentation for the specific appbar message sent. Links to those documents are given in the See Also section.</para>
        /// </returns>
        /// <remarks>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/nf-shellapi-shappbarmessage">Learn more about this API from docs.microsoft.com</see>.</para>
        /// </remarks>
        [DllImport("Shell32", ExactSpelling = true)]
        [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
        internal static extern unsafe nuint SHAppBarMessage(uint dwMessage, APPBARDATA32* pData);

        /// <summary>Sends an appbar message to the system.</summary>
        /// <param name="dwMessage">Type: <b>DWORD</b></param>
        /// <param name="pData">
        /// <para>Type: <b>PAPPBARDATA</b></para>
        /// <para>A pointer to an <a href="https://docs.microsoft.com/windows/desktop/api/shellapi/ns-shellapi-appbardata">APPBARDATA</a> structure. The content of the structure on entry and on exit depends on the value set in the <i>dwMessage</i> parameter. See the individual message pages for specifics.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/nf-shellapi-shappbarmessage#parameters">Read more on docs.microsoft.com</see>.</para>
        /// </param>
        /// <returns>
        /// <para>Type: <b>UINT_PTR</b></para>
        /// <para>This function returns a message-dependent value. For more information, see the Windows SDK documentation for the specific appbar message sent. Links to those documents are given in the See Also section.</para>
        /// </returns>
        /// <remarks>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/nf-shellapi-shappbarmessage">Learn more about this API from docs.microsoft.com</see>.</para>
        /// </remarks>
        [DllImport("Shell32", ExactSpelling = true)]
        [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
        internal static extern unsafe nuint SHAppBarMessage(uint dwMessage, APPBARDATA64* pData);

        /// <inheritdoc cref = "Shell_NotifyIcon(uint, NOTIFYICONDATAW32*)"/>
        internal static unsafe bool Shell_NotifyIcon(uint dwMessage, in NOTIFYICONDATAW32 lpData)
        {
            fixed (NOTIFYICONDATAW32* lpDataLocal = &lpData)
            {
                bool __result = PInvoke.Shell_NotifyIcon(dwMessage, lpDataLocal);
                return __result;
            }
        }
        /// <inheritdoc cref = "Shell_NotifyIcon(uint, NOTIFYICONDATAW32*)"/>
        internal static unsafe bool Shell_NotifyIcon(uint dwMessage, in NOTIFYICONDATAW64 lpData)
        {
            fixed (NOTIFYICONDATAW64* lpDataLocal = &lpData)
            {
                bool __result = PInvoke.Shell_NotifyIcon(dwMessage, lpDataLocal);
                return __result;
            }
        }

        /// <summary>Sends a message to the taskbar's status area.</summary>
        /// <param name="dwMessage">Type: <b>DWORD</b></param>
        /// <param name="lpData">
        /// <para>Type: <b>PNOTIFYICONDATA</b></para>
        /// <para>A pointer to a <a href="https://docs.microsoft.com/windows/desktop/api/shellapi/ns-shellapi-notifyicondataa">NOTIFYICONDATA</a> structure. The content of the structure depends on the value of <i>dwMessage</i>. It can define an icon to add to the notification area, cause that icon to display a notification, or identify an icon to modify or delete.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/nf-shellapi-shell_notifyiconw#parameters">Read more on docs.microsoft.com</see>.</para>
        /// </param>
        /// <returns>
        /// <para>Type: <b>BOOL</b></para>
        /// <para>Returns <b>TRUE</b> if successful, or <b>FALSE</b> otherwise. If <i>dwMessage</i> is set to NIM_SETVERSION, the function returns <b>TRUE</b> if the version was successfully changed, or <b>FALSE</b> if the requested version is not supported.</para>
        /// </returns>
        /// <remarks>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/nf-shellapi-shell_notifyiconw">Learn more about this API from docs.microsoft.com</see>.</para>
        /// </remarks>
        [DllImport("Shell32", ExactSpelling = true, EntryPoint = "Shell_NotifyIconW")]
        [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
        internal static extern unsafe bool Shell_NotifyIcon(uint dwMessage, NOTIFYICONDATAW32* lpData);

        /// <summary>Sends a message to the taskbar's status area.</summary>
        /// <param name="dwMessage">Type: <b>DWORD</b></param>
        /// <param name="lpData">
        /// <para>Type: <b>PNOTIFYICONDATA</b></para>
        /// <para>A pointer to a <a href="https://docs.microsoft.com/windows/desktop/api/shellapi/ns-shellapi-notifyicondataa">NOTIFYICONDATA</a> structure. The content of the structure depends on the value of <i>dwMessage</i>. It can define an icon to add to the notification area, cause that icon to display a notification, or identify an icon to modify or delete.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/nf-shellapi-shell_notifyiconw#parameters">Read more on docs.microsoft.com</see>.</para>
        /// </param>
        /// <returns>
        /// <para>Type: <b>BOOL</b></para>
        /// <para>Returns <b>TRUE</b> if successful, or <b>FALSE</b> otherwise. If <i>dwMessage</i> is set to NIM_SETVERSION, the function returns <b>TRUE</b> if the version was successfully changed, or <b>FALSE</b> if the requested version is not supported.</para>
        /// </returns>
        /// <remarks>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/nf-shellapi-shell_notifyiconw">Learn more about this API from docs.microsoft.com</see>.</para>
        /// </remarks>
        [DllImport("Shell32", ExactSpelling = true, EntryPoint = "Shell_NotifyIconW")]
        [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
        internal static extern unsafe bool Shell_NotifyIcon(uint dwMessage, NOTIFYICONDATAW64* lpData);
    }

    /// <summary>Contains information about a system appbar message.</summary>
    /// <remarks>
    /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-appbardata">Learn more about this API from docs.microsoft.com</see>.</para>
    /// </remarks>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    internal partial struct APPBARDATA32
    {
        /// <summary>
        /// <para>Type: <b>DWORD</b></para>
        /// <para>The size of the structure, in bytes.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-appbardata#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint cbSize;
        /// <summary>
        /// <para>Type: <b>HWND</b></para>
        /// <para>The handle to the appbar window. Not all messages use this member. See the individual message page to see if you need to provide an <b>hWind</b> value.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-appbardata#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal HWND hWnd;
        /// <summary>
        /// <para>Type: <b>UINT</b></para>
        /// <para>An application-defined message identifier. The application uses the specified identifier for notification messages that it sends to the appbar identified by the <b>hWnd</b> member. This member is used when sending the <a href="https://docs.microsoft.com/windows/desktop/shell/abm-new">ABM_NEW</a> message.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-appbardata#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint uCallbackMessage;
        /// <summary>
        /// <para>Type: <b>UINT</b> A value that specifies an edge of the screen. This member is used when sending one of these messages: </para>
        /// <para>This doc was truncated.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-appbardata#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint uEdge;
        /// <summary>
        /// <para>Type: <b><a href="https://docs.microsoft.com/windows/desktop/api/windef/ns-windef-rect">RECT</a></b> A <a href="https://docs.microsoft.com/windows/desktop/api/windef/ns-windef-rect">RECT</a> structure whose use varies depending on the message:</para>
        /// <para></para>
        /// <para>This doc was truncated.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-appbardata#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal RECT rc;
        /// <summary>
        /// <para>Type: <b>LPARAM</b> A message-dependent value. This member is used with these messages: </para>
        /// <para>This doc was truncated.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-appbardata#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal LPARAM lParam;
    }

    /// <summary>Contains information about a system appbar message.</summary>
    /// <remarks>
    /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-appbardata">Learn more about this API from docs.microsoft.com</see>.</para>
    /// </remarks>
    internal partial struct APPBARDATA64
    {
        /// <summary>
        /// <para>Type: <b>DWORD</b></para>
        /// <para>The size of the structure, in bytes.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-appbardata#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint cbSize;
        /// <summary>
        /// <para>Type: <b>HWND</b></para>
        /// <para>The handle to the appbar window. Not all messages use this member. See the individual message page to see if you need to provide an <b>hWind</b> value.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-appbardata#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal HWND hWnd;
        /// <summary>
        /// <para>Type: <b>UINT</b></para>
        /// <para>An application-defined message identifier. The application uses the specified identifier for notification messages that it sends to the appbar identified by the <b>hWnd</b> member. This member is used when sending the <a href="https://docs.microsoft.com/windows/desktop/shell/abm-new">ABM_NEW</a> message.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-appbardata#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint uCallbackMessage;
        /// <summary>
        /// <para>Type: <b>UINT</b> A value that specifies an edge of the screen. This member is used when sending one of these messages: </para>
        /// <para>This doc was truncated.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-appbardata#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint uEdge;
        /// <summary>
        /// <para>Type: <b><a href="https://docs.microsoft.com/windows/desktop/api/windef/ns-windef-rect">RECT</a></b> A <a href="https://docs.microsoft.com/windows/desktop/api/windef/ns-windef-rect">RECT</a> structure whose use varies depending on the message:</para>
        /// <para></para>
        /// <para>This doc was truncated.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-appbardata#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal RECT rc;
        /// <summary>
        /// <para>Type: <b>LPARAM</b> A message-dependent value. This member is used with these messages: </para>
        /// <para>This doc was truncated.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-appbardata#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal LPARAM lParam;
    }
    internal struct __ushort_128
    {
        internal ushort _0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19, _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38, _39, _40, _41, _42, _43, _44, _45, _46, _47, _48, _49, _50, _51, _52, _53, _54, _55, _56, _57, _58, _59, _60, _61, _62, _63, _64, _65, _66, _67, _68, _69, _70, _71, _72, _73, _74, _75, _76, _77, _78, _79, _80, _81, _82, _83, _84, _85, _86, _87, _88, _89, _90, _91, _92, _93, _94, _95, _96, _97, _98, _99, _100, _101, _102, _103, _104, _105, _106, _107, _108, _109, _110, _111, _112, _113, _114, _115, _116, _117, _118, _119, _120, _121, _122, _123, _124, _125, _126, _127;
        /// <summary>Always <c>128</c>.</summary>
        internal int Length => 128;
        /// <summary>
        /// Gets a ref to an individual element of the inline array.
        /// ⚠ Important ⚠: When this struct is on the stack, do not let the returned reference outlive the stack frame that defines it.
        /// </summary>
        internal ref ushort this[int index] => ref AsSpan()[index];
        /// <summary>
        /// Gets this inline array as a span.
        /// </summary>
        /// <remarks>
        /// ⚠ Important ⚠: When this struct is on the stack, do not let the returned span outlive the stack frame that defines it.
        /// </remarks>
        internal Span<ushort> AsSpan() => MemoryMarshal.CreateSpan(ref _0, 128);
    }

    /// <summary>Contains information that the system needs to display notifications in the notification area. Used by Shell_NotifyIcon.</summary>
    /// <remarks>
    /// <para>See <a href="https://msdn.microsoft.com/library/aa511497.aspx">Notifications</a> in the Windows User Experience Interaction Guidelines for more information on notification UI and content best practices.</para>
    /// <para>If you set the <b>NIF_INFO</b> flag in the <b>uFlags</b> member, the balloon-style notification is used. For more discussion of these notifications, see <a href="https://docs.microsoft.com/windows/desktop/Controls/using-tooltip-contro">Balloon tooltips</a>.</para>
    /// <para>No more than one balloon notification at a time can be displayed for the taskbar. If an application attempts to display a notification when one is already being displayed, the new notification is queued and displayed when the older notification goes away. In versions of Windows before Windows Vista, the new notification would not appear until the existing notification has been visible for at least the system minimum timeout length, regardless of the original notification's <b>uTimeout</b> value. If the user does not appear to be using the computer, the system does not count this time toward the timeout.</para>
    /// <para>Several members of this structure are only supported for Windows 2000 and later. To enable these members, include one of the following lines in your header:</para>
    /// <para></para>
    /// <para>This doc was truncated.</para>
    /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#">Read more on docs.microsoft.com</see>.</para>
    /// </remarks>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    internal partial struct NOTIFYICONDATAW32
    {
        /// <summary>
        /// <para>Type: <b>DWORD</b></para>
        /// <para>The size of this structure, in bytes.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint cbSize;
        /// <summary>
        /// <para>Type: <b>HWND</b></para>
        /// <para>A handle to the window that receives notifications associated with an icon in the notification area.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal HWND hWnd;
        /// <summary>
        /// <para>Type: <b>UINT</b></para>
        /// <para>The application-defined identifier of the taskbar icon. The Shell uses either (<b>hWnd</b> plus <b>uID</b>) or <b>guidItem</b> to identify which icon to operate on when <a href="https://docs.microsoft.com/windows/desktop/api/shellapi/nf-shellapi-shell_notifyicona">Shell_NotifyIcon</a> is invoked. You can have multiple icons associated with a single <b>hWnd</b> by assigning each a different <b>uID</b>. If <b>guidItem</b> is specified, <b>uID</b> is ignored.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint uID;
        /// <summary>Type: <b>UINT</b></summary>
        internal uint uFlags;
        /// <summary>
        /// <para>Type: <b>UINT</b></para>
        /// <para>An application-defined message identifier. The system uses this identifier to send notification messages to the window identified in <b>hWnd</b>. These notification messages are sent when a mouse event or hover occurs in the bounding rectangle of the icon, when the icon is selected or activated with the keyboard, or when those actions occur in the balloon notification.</para>
        /// <para>When the <b>uVersion</b> member is either 0 or NOTIFYICON_VERSION, the <i>wParam</i> parameter of the message contains the identifier of the taskbar icon in which the event occurred. This identifier can be 32 bits in length. The <i>lParam</i> parameter holds the mouse or keyboard message associated with the event. For example, when the pointer moves over a taskbar icon, <i>lParam</i> is set to <a href="https://docs.microsoft.com/windows/desktop/inputdev/wm-mousemove">WM_MOUSEMOVE</a>.</para>
        /// <para>When the <b>uVersion</b> member is NOTIFYICON_VERSION_4, applications continue to receive notification events in the form of application-defined messages through the <b>uCallbackMessage</b> member, but the interpretation of the <i>lParam</i> and <i>wParam</i> parameters of that message is changed as follows:</para>
        /// <para></para>
        /// <para>This doc was truncated.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint uCallbackMessage;
        /// <summary>
        /// <para>Type: <b>HICON</b></para>
        /// <para>A handle to the icon to be added, modified, or deleted. Windows XP and later support icons of up to 32 BPP.</para>
        /// <para>If only a 16x16 pixel icon is provided, it is scaled to a larger size in a system set to a high dpi value. This can lead to an unattractive result. It is recommended that you provide both a 16x16 pixel icon and a 32x32 icon in your resource file. Use <a href="https://docs.microsoft.com/windows/desktop/api/commctrl/nf-commctrl-loadiconmetric">LoadIconMetric</a> to ensure that the correct icon is loaded and scaled appropriately. See Remarks for a code example.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal HICON hIcon;
        /// <summary>
        /// <para>Type: <b>TCHAR[64]</b></para>
        /// <para>A null-terminated string that specifies the text for a standard tooltip. It can have a maximum of 64 characters, including the terminating null character.</para>
        /// <para>For Windows 2000 and later, <b>szTip</b> can have a maximum of 128 characters, including the terminating null character.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal __ushort_128 szTip;
        /// <summary>Type: <b>DWORD</b></summary>
        internal uint dwState;
        /// <summary>
        /// <para>Type: <b>DWORD</b></para>
        /// <para><b>Windows 2000 and later</b>. A value that specifies which bits of the <b>dwState</b> member are retrieved or modified. The possible values are the same as those for <b>dwState</b>. For example, setting this member to <b>NIS_HIDDEN</b> causes only the item's hidden state to be modified while the icon sharing bit is ignored regardless of its value.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint dwStateMask;
        /// <summary>
        /// <para>Type: <b>TCHAR[256]</b></para>
        /// <para><b>Windows 2000 and later</b>. A null-terminated string that specifies the text to display in a balloon notification. It can have a maximum of 256 characters, including the terminating null character, but should be restricted to 200 characters in English to accommodate localization. To remove the balloon notification from the UI, either delete the icon (with <a href="https://docs.microsoft.com/windows/desktop/api/shellapi/nf-shellapi-shell_notifyicona">NIM_DELETE</a>) or set the <b>NIF_INFO</b> flag in <b>uFlags</b> and set <b>szInfo</b> to an empty string.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal __ushort_256 szInfo;
        internal _Anonymous_e__Union Anonymous;
        /// <summary>
        /// <para>Type: <b>TCHAR[64]</b></para>
        /// <para><b>Windows 2000 and later</b>. A null-terminated string that specifies a title for a balloon notification. This title appears in a larger font immediately above the text. It can have a maximum of 64 characters, including the terminating null character, but should be restricted to 48 characters in English to accommodate localization.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal __ushort_64 szInfoTitle;
        /// <summary>
        /// <para>Type: <b>DWORD</b></para>
        /// <para><b>Windows 2000 and later</b>. Flags that can be set to modify the behavior and appearance of a balloon notification. The icon is placed to the left of the title. If the <b>szInfoTitle</b> member is zero-length, the icon is not shown.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint dwInfoFlags;
        /// <summary>
        /// <para>Type: <b>GUID</b> <b>Windows XP and later</b>.</para>
        /// <para></para>
        /// <para>This doc was truncated.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal global::System.Guid guidItem;
        /// <summary>
        /// <para>Type: <b>HICON</b></para>
        /// <para><b>Windows Vista and later</b>. The handle of a customized notification icon provided by the application that should be used independently of the notification area icon. If this member is non-NULL and the NIIF_USER flag is set in the <b>dwInfoFlags</b> member, this icon is used as the notification icon. If this member is <b>NULL</b>, the legacy behavior is carried out.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal HICON hBalloonIcon;
     

        internal struct __ushort_256
        {
            internal ushort _0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19, _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38, _39, _40, _41, _42, _43, _44, _45, _46, _47, _48, _49, _50, _51, _52, _53, _54, _55, _56, _57, _58, _59, _60, _61, _62, _63, _64, _65, _66, _67, _68, _69, _70, _71, _72, _73, _74, _75, _76, _77, _78, _79, _80, _81, _82, _83, _84, _85, _86, _87, _88, _89, _90, _91, _92, _93, _94, _95, _96, _97, _98, _99, _100, _101, _102, _103, _104, _105, _106, _107, _108, _109, _110, _111, _112, _113, _114, _115, _116, _117, _118, _119, _120, _121, _122, _123, _124, _125, _126, _127, _128, _129, _130, _131, _132, _133, _134, _135, _136, _137, _138, _139, _140, _141, _142, _143, _144, _145, _146, _147, _148, _149, _150, _151, _152, _153, _154, _155, _156, _157, _158, _159, _160, _161, _162, _163, _164, _165, _166, _167, _168, _169, _170, _171, _172, _173, _174, _175, _176, _177, _178, _179, _180, _181, _182, _183, _184, _185, _186, _187, _188, _189, _190, _191, _192, _193, _194, _195, _196, _197, _198, _199, _200, _201, _202, _203, _204, _205, _206, _207, _208, _209, _210, _211, _212, _213, _214, _215, _216, _217, _218, _219, _220, _221, _222, _223, _224, _225, _226, _227, _228, _229, _230, _231, _232, _233, _234, _235, _236, _237, _238, _239, _240, _241, _242, _243, _244, _245, _246, _247, _248, _249, _250, _251, _252, _253, _254, _255;
            /// <summary>Always <c>256</c>.</summary>
            internal int Length => 256;
            /// <summary>
            /// Gets a ref to an individual element of the inline array.
            /// ⚠ Important ⚠: When this struct is on the stack, do not let the returned reference outlive the stack frame that defines it.
            /// </summary>
            internal ref ushort this[int index] => ref AsSpan()[index];
            /// <summary>
            /// Gets this inline array as a span.
            /// </summary>
            /// <remarks>
            /// ⚠ Important ⚠: When this struct is on the stack, do not let the returned span outlive the stack frame that defines it.
            /// </remarks>
            internal Span<ushort> AsSpan() => MemoryMarshal.CreateSpan(ref _0, 256);
        }

        [StructLayout(LayoutKind.Explicit, Pack = 1)]
        internal partial struct _Anonymous_e__Union
        {
            [FieldOffset(0)]
            internal uint uTimeout;
            [FieldOffset(0)]
            internal uint uVersion;
        }
    }


    internal struct __ushort_64
    {
        internal ushort _0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19, _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38, _39, _40, _41, _42, _43, _44, _45, _46, _47, _48, _49, _50, _51, _52, _53, _54, _55, _56, _57, _58, _59, _60, _61, _62, _63;
        /// <summary>Always <c>64</c>.</summary>
        internal int Length => 64;
        /// <summary>
        /// Gets a ref to an individual element of the inline array.
        /// ⚠ Important ⚠: When this struct is on the stack, do not let the returned reference outlive the stack frame that defines it.
        /// </summary>
        internal ref ushort this[int index] => ref AsSpan()[index];
        /// <summary>
        /// Gets this inline array as a span.
        /// </summary>
        /// <remarks>
        /// ⚠ Important ⚠: When this struct is on the stack, do not let the returned span outlive the stack frame that defines it.
        /// </remarks>
        internal Span<ushort> AsSpan() => MemoryMarshal.CreateSpan(ref _0, 64);
    }


    /// <summary>Contains information that the system needs to display notifications in the notification area. Used by Shell_NotifyIcon.</summary>
    /// <remarks>
    /// <para>See <a href="https://msdn.microsoft.com/library/aa511497.aspx">Notifications</a> in the Windows User Experience Interaction Guidelines for more information on notification UI and content best practices.</para>
    /// <para>If you set the <b>NIF_INFO</b> flag in the <b>uFlags</b> member, the balloon-style notification is used. For more discussion of these notifications, see <a href="https://docs.microsoft.com/windows/desktop/Controls/using-tooltip-contro">Balloon tooltips</a>.</para>
    /// <para>No more than one balloon notification at a time can be displayed for the taskbar. If an application attempts to display a notification when one is already being displayed, the new notification is queued and displayed when the older notification goes away. In versions of Windows before Windows Vista, the new notification would not appear until the existing notification has been visible for at least the system minimum timeout length, regardless of the original notification's <b>uTimeout</b> value. If the user does not appear to be using the computer, the system does not count this time toward the timeout.</para>
    /// <para>Several members of this structure are only supported for Windows 2000 and later. To enable these members, include one of the following lines in your header:</para>
    /// <para></para>
    /// <para>This doc was truncated.</para>
    /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#">Read more on docs.microsoft.com</see>.</para>
    /// </remarks>
    internal partial struct NOTIFYICONDATAW64
    {
        /// <summary>
        /// <para>Type: <b>DWORD</b></para>
        /// <para>The size of this structure, in bytes.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint cbSize;
        /// <summary>
        /// <para>Type: <b>HWND</b></para>
        /// <para>A handle to the window that receives notifications associated with an icon in the notification area.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal HWND hWnd;
        /// <summary>
        /// <para>Type: <b>UINT</b></para>
        /// <para>The application-defined identifier of the taskbar icon. The Shell uses either (<b>hWnd</b> plus <b>uID</b>) or <b>guidItem</b> to identify which icon to operate on when <a href="https://docs.microsoft.com/windows/desktop/api/shellapi/nf-shellapi-shell_notifyicona">Shell_NotifyIcon</a> is invoked. You can have multiple icons associated with a single <b>hWnd</b> by assigning each a different <b>uID</b>. If <b>guidItem</b> is specified, <b>uID</b> is ignored.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint uID;
        /// <summary>Type: <b>UINT</b></summary>
        internal uint uFlags;
        /// <summary>
        /// <para>Type: <b>UINT</b></para>
        /// <para>An application-defined message identifier. The system uses this identifier to send notification messages to the window identified in <b>hWnd</b>. These notification messages are sent when a mouse event or hover occurs in the bounding rectangle of the icon, when the icon is selected or activated with the keyboard, or when those actions occur in the balloon notification.</para>
        /// <para>When the <b>uVersion</b> member is either 0 or NOTIFYICON_VERSION, the <i>wParam</i> parameter of the message contains the identifier of the taskbar icon in which the event occurred. This identifier can be 32 bits in length. The <i>lParam</i> parameter holds the mouse or keyboard message associated with the event. For example, when the pointer moves over a taskbar icon, <i>lParam</i> is set to <a href="https://docs.microsoft.com/windows/desktop/inputdev/wm-mousemove">WM_MOUSEMOVE</a>.</para>
        /// <para>When the <b>uVersion</b> member is NOTIFYICON_VERSION_4, applications continue to receive notification events in the form of application-defined messages through the <b>uCallbackMessage</b> member, but the interpretation of the <i>lParam</i> and <i>wParam</i> parameters of that message is changed as follows:</para>
        /// <para></para>
        /// <para>This doc was truncated.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint uCallbackMessage;
        /// <summary>
        /// <para>Type: <b>HICON</b></para>
        /// <para>A handle to the icon to be added, modified, or deleted. Windows XP and later support icons of up to 32 BPP.</para>
        /// <para>If only a 16x16 pixel icon is provided, it is scaled to a larger size in a system set to a high dpi value. This can lead to an unattractive result. It is recommended that you provide both a 16x16 pixel icon and a 32x32 icon in your resource file. Use <a href="https://docs.microsoft.com/windows/desktop/api/commctrl/nf-commctrl-loadiconmetric">LoadIconMetric</a> to ensure that the correct icon is loaded and scaled appropriately. See Remarks for a code example.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal HICON hIcon;
        /// <summary>
        /// <para>Type: <b>TCHAR[64]</b></para>
        /// <para>A null-terminated string that specifies the text for a standard tooltip. It can have a maximum of 64 characters, including the terminating null character.</para>
        /// <para>For Windows 2000 and later, <b>szTip</b> can have a maximum of 128 characters, including the terminating null character.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal __ushort_128 szTip;
        /// <summary>Type: <b>DWORD</b></summary>
        internal uint dwState;
        /// <summary>
        /// <para>Type: <b>DWORD</b></para>
        /// <para><b>Windows 2000 and later</b>. A value that specifies which bits of the <b>dwState</b> member are retrieved or modified. The possible values are the same as those for <b>dwState</b>. For example, setting this member to <b>NIS_HIDDEN</b> causes only the item's hidden state to be modified while the icon sharing bit is ignored regardless of its value.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint dwStateMask;
        /// <summary>
        /// <para>Type: <b>TCHAR[256]</b></para>
        /// <para><b>Windows 2000 and later</b>. A null-terminated string that specifies the text to display in a balloon notification. It can have a maximum of 256 characters, including the terminating null character, but should be restricted to 200 characters in English to accommodate localization. To remove the balloon notification from the UI, either delete the icon (with <a href="https://docs.microsoft.com/windows/desktop/api/shellapi/nf-shellapi-shell_notifyicona">NIM_DELETE</a>) or set the <b>NIF_INFO</b> flag in <b>uFlags</b> and set <b>szInfo</b> to an empty string.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal __ushort_256 szInfo;
        internal _Anonymous_e__Union Anonymous;
        /// <summary>
        /// <para>Type: <b>TCHAR[64]</b></para>
        /// <para><b>Windows 2000 and later</b>. A null-terminated string that specifies a title for a balloon notification. This title appears in a larger font immediately above the text. It can have a maximum of 64 characters, including the terminating null character, but should be restricted to 48 characters in English to accommodate localization.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal __ushort_64 szInfoTitle;
        /// <summary>
        /// <para>Type: <b>DWORD</b></para>
        /// <para><b>Windows 2000 and later</b>. Flags that can be set to modify the behavior and appearance of a balloon notification. The icon is placed to the left of the title. If the <b>szInfoTitle</b> member is zero-length, the icon is not shown.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal uint dwInfoFlags;
        /// <summary>
        /// <para>Type: <b>GUID</b> <b>Windows XP and later</b>.</para>
        /// <para></para>
        /// <para>This doc was truncated.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal global::System.Guid guidItem;
        /// <summary>
        /// <para>Type: <b>HICON</b></para>
        /// <para><b>Windows Vista and later</b>. The handle of a customized notification icon provided by the application that should be used independently of the notification area icon. If this member is non-NULL and the NIIF_USER flag is set in the <b>dwInfoFlags</b> member, this icon is used as the notification icon. If this member is <b>NULL</b>, the legacy behavior is carried out.</para>
        /// <para><see href="https://docs.microsoft.com/windows/win32/api//shellapi/ns-shellapi-notifyicondataw#members">Read more on docs.microsoft.com</see>.</para>
        /// </summary>
        internal HICON hBalloonIcon;
       
        internal struct __ushort_256
        {
            internal ushort _0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19, _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38, _39, _40, _41, _42, _43, _44, _45, _46, _47, _48, _49, _50, _51, _52, _53, _54, _55, _56, _57, _58, _59, _60, _61, _62, _63, _64, _65, _66, _67, _68, _69, _70, _71, _72, _73, _74, _75, _76, _77, _78, _79, _80, _81, _82, _83, _84, _85, _86, _87, _88, _89, _90, _91, _92, _93, _94, _95, _96, _97, _98, _99, _100, _101, _102, _103, _104, _105, _106, _107, _108, _109, _110, _111, _112, _113, _114, _115, _116, _117, _118, _119, _120, _121, _122, _123, _124, _125, _126, _127, _128, _129, _130, _131, _132, _133, _134, _135, _136, _137, _138, _139, _140, _141, _142, _143, _144, _145, _146, _147, _148, _149, _150, _151, _152, _153, _154, _155, _156, _157, _158, _159, _160, _161, _162, _163, _164, _165, _166, _167, _168, _169, _170, _171, _172, _173, _174, _175, _176, _177, _178, _179, _180, _181, _182, _183, _184, _185, _186, _187, _188, _189, _190, _191, _192, _193, _194, _195, _196, _197, _198, _199, _200, _201, _202, _203, _204, _205, _206, _207, _208, _209, _210, _211, _212, _213, _214, _215, _216, _217, _218, _219, _220, _221, _222, _223, _224, _225, _226, _227, _228, _229, _230, _231, _232, _233, _234, _235, _236, _237, _238, _239, _240, _241, _242, _243, _244, _245, _246, _247, _248, _249, _250, _251, _252, _253, _254, _255;
            /// <summary>Always <c>256</c>.</summary>
            internal int Length => 256;
            /// <summary>
            /// Gets a ref to an individual element of the inline array.
            /// ⚠ Important ⚠: When this struct is on the stack, do not let the returned reference outlive the stack frame that defines it.
            /// </summary>
            internal ref ushort this[int index] => ref AsSpan()[index];
            /// <summary>
            /// Gets this inline array as a span.
            /// </summary>
            /// <remarks>
            /// ⚠ Important ⚠: When this struct is on the stack, do not let the returned span outlive the stack frame that defines it.
            /// </remarks>
            internal Span<ushort> AsSpan() => MemoryMarshal.CreateSpan(ref _0, 256);
        }

        [StructLayout(LayoutKind.Explicit)]
        internal partial struct _Anonymous_e__Union
        {
            [FieldOffset(0)]
            internal uint uTimeout;
            [FieldOffset(0)]
            internal uint uVersion;
        }
    }
}