# Troubleshooting Blank Page

If you're seeing a blank page at http://localhost:3000, follow these steps:

## 1. Check Browser Console

Open your browser's Developer Tools (F12 or Cmd+Option+I) and check:

- **Console tab**: Look for JavaScript errors (red text)
- **Network tab**: Check if files are loading (should see 200 status codes)
- **Elements tab**: Check if `<div id="root">` exists in the HTML

## 2. Common Issues

### Issue: JavaScript Errors in Console

If you see errors like:
- `Cannot read property 'X' of undefined`
- `Module not found`
- `TypeError: ...`

**Solution**: Check the error message and file. Common causes:
- Missing dependencies: Run `npm install`
- Import errors: Check file paths
- Type errors: Run `npm run build` to see TypeScript errors

### Issue: Network Errors

If files aren't loading:
- Check if dev server is running: `npm run dev`
- Check if port 3000 is available
- Try a different port: Edit `vite.config.ts` and change port

### Issue: Root Element Not Found

If you see "root element not found":
- Check `index.html` has `<div id="root"></div>`
- Check `main.tsx` is importing correctly

### Issue: Infinite Redirect Loop

If you see the page constantly refreshing:
- Check browser console for redirect errors
- Clear localStorage: `localStorage.clear()` in console
- Check `ProtectedRoute` component logic

## 3. Quick Debug Steps

### Step 1: Verify Dev Server is Running

```bash
cd applications/heatpump-web
npm run dev
```

You should see:
```
  VITE v5.x.x  ready in xxx ms

  ➜  Local:   http://localhost:3000/
  ➜  Network: use --host to expose
```

### Step 2: Check Browser Console

1. Open http://localhost:3000
2. Press F12 to open DevTools
3. Go to Console tab
4. Look for any red error messages
5. Share the error message if you see one

### Step 3: Verify Files Are Loading

1. Open DevTools → Network tab
2. Refresh the page
3. Check if these files load with 200 status:
   - `index.html`
   - `main.tsx` (or bundled JS)
   - `index.css`

### Step 4: Test Simple Render

If the page is still blank, try temporarily simplifying `App.tsx`:

```tsx
function App() {
  return <div>Hello World</div>;
}
```

If this works, the issue is in the routing or component imports.

## 4. Check These Files

Make sure these files exist and are correct:
- ✅ `index.html` - Has `<div id="root"></div>`
- ✅ `src/main.tsx` - Imports App and renders to root
- ✅ `src/App.tsx` - Main app component
- ✅ `src/vite-env.d.ts` - Vite type definitions (just created)

## 5. Common Fixes

### Fix: Clear Cache and Reinstall

```bash
cd applications/heatpump-web
rm -rf node_modules package-lock.json
npm install
npm run dev
```

### Fix: Check Port Conflicts

```bash
# Check if port 3000 is in use
lsof -i :3000

# Kill process if needed
kill -9 <PID>

# Or change port in vite.config.ts
```

### Fix: Check Environment Variables

Make sure `.env` file exists:
```bash
cat .env
# Should show: VITE_API_URL=http://localhost:8080
```

## 6. Still Not Working?

If none of the above works:

1. **Share the browser console errors** - This is the most helpful
2. **Check if the dev server shows any errors** in the terminal
3. **Try building for production** to see if there are build errors:
   ```bash
   npm run build
   npm run preview
   ```

## 7. Expected Behavior

When working correctly:
1. Visit http://localhost:3000
2. Should redirect to `/login` (if not authenticated) or `/dashboard` (if authenticated)
3. Login page should show a form with username/password fields
4. After login, should redirect to dashboard with cards showing data
